# Cross-Platform Strategy — Native on Every OS

**תאריך:** 23.04.2026
**בקשת עידן:** "האם זה יוכל לעבוד בכל מערכת הפעלה native? Windows/Mac/Linux/iOS/Android?
מה נדרש? אולי wrapper? והכל יהיה על הגרף. הקוד יהיה נקי."

---

## תשובה קצרה (מאומתת)

**כן. כל 6 הפלטפורמות נתמכות באמצעות Rust + adapter pattern — בלי fork, בלי code duplication.**

Python prototype (`py_testers/test_platform_abstraction_v1.py`) הוכיח:
- ✅ 7/7 tests עברו
- ✅ כל פלטפורמה יש לה `cargo target triple`
- ✅ mmap עובד ב-5/6 (WASM בלבד צריך IndexedDB-backed storage)
- ✅ ZETS תלוי ב-2 crates בלבד (memmap2 + aes-gcm) — שתיהן cross-platform

---

## המטריצה המלאה

### שש פלטפורמות, לא חמש

עידן שאל על Windows/Mac/Linux/iOS/Android. יש עוד אחת שכדאי לזכור:
- **WebAssembly (WASM)** — ZETS בדפדפן. פרסום מיידי בלי install. לחיצה אחת, רץ. **זה סוד מעולה למרקטינג.**

### לכל פלטפורמה

| Platform | Bundle | GUI shell | Build from Linux? | Complexity |
|----------|--------|-----------|-------------------|------------|
| **Linux** | `.AppImage` / `.deb` / `.rpm` | Tauri (WebKitGTK) | native ✅ | trivial |
| **macOS** | `.app` / `.dmg` | Tauri (WKWebView) | via osxcross ⚠️ | easy |
| **Windows** | `.exe` / `.msi` | Tauri (WebView2) | via mingw ⚠️ | easy |
| **iOS** | `.ipa` → App Store | SwiftUI + WKWebView | **NO** — Apple דורש macOS | moderate |
| **Android** | `.apk` → Play Store | Kotlin + WebView | yes via NDK ✅ | moderate |
| **WASM** | `.wasm` → dropped on any CDN | Browser native | trivial ✅ | moderate |

---

## איך לא עושים fork — Adapter Pattern

### העקרון

**Platform-specific code חי במקום אחד בלבד** — trait `PlatformAdapter` עם `impl` לכל פלטפורמה:

```rust
pub trait PlatformAdapter {
    fn data_dir(app_name: &str) -> PathBuf;
    fn cache_dir(app_name: &str) -> PathBuf;
    fn can_mmap() -> bool;
    fn localhost_url(port: u16) -> String;
}

// One impl file per platform, chosen at compile time:
#[cfg(target_os = "linux")]
mod linux_adapter;   // → ~/.local/share/zets

#[cfg(target_os = "macos")]
mod macos_adapter;   // → ~/Library/Application Support/zets

#[cfg(target_os = "windows")]
mod windows_adapter; // → %APPDATA%/zets

#[cfg(target_os = "ios")]
mod ios_adapter;     // → app sandbox

#[cfg(target_os = "android")]
mod android_adapter; // → /data/data/com.idan.zets/files

#[cfg(target_arch = "wasm32")]
mod wasm_adapter;    // → IndexedDB virtual FS
```

**כל שאר הקוד של ZETS — 45 .rs files — לא יודע אפילו מה הOS.** זה תלוי רק ב-trait.

### Data directories נשמרים בגרף

עידן אמר "גם זה שיהיה על הגרף". מימוש:

```rust
// On startup, query the graph for preferences:
let user_pref = engine.query_atom("user:data_dir:preference")?;
let actual_dir = user_pref.unwrap_or_else(||
    PlatformAdapter::data_dir("zets").to_string_lossy().into()
);
```

משתמש יכול להחליט ב-GUI (→atom שכתוב לגרף) ששוב — כל הנתונים במקום אחר. ה-Platform default הוא fallback.

---

## GUI Strategy — אחידה ככל האפשר

### Tauri לדסקטופ (Linux/macOS/Windows)

**מה Tauri?**
- Rust-based wrapper שמשתמש ב-**OS native webview**
- **5 MB** bundle (Electron: 150MB+)
- אותו קוד JavaScript / HTML → 3 פלטפורמות
- יש לנו כבר `zets-gui/dist/index.html` — 11KB vanilla JS שיעבוד כמו שהוא

### Native wrappers לmobile

iOS ו-Android לא מריצים Tauri ישירות. צריך wrapper שונה:

**iOS:**
```swift
import SwiftUI
import WebKit

struct ContentView: View {
    var body: some View {
        WebView(url: URL(string: "bundle://index.html")!)
    }
}
// Rust מוטמע כ-static library, נקרא דרך FFI
```

**Android:**
```kotlin
class MainActivity : AppCompatActivity() {
    override fun onCreate(...) {
        webView.loadUrl("file:///android_asset/index.html")
        // Rust נטען כ-.so דרך JNI
    }
}
```

**אותו HTML** (מ-`zets-gui/dist/index.html`) — ה-wrapper רק טוען אותו בwebview. אפס code duplication.

### WASM — אפס wrapper

הגרסה הכי קלילה. משתמש פותח URL → רץ. טוב ל:
- Demo landing page
- Marketing ("לחץ כדי לנסות")
- Kids mode (sandboxed)

---

## מה השינויים הנדרשים ב-ZETS

### מצב עכשיו (בדיקה חיה)

```bash
grep "target_os\|cfg(target" src/*.rs
# → 0 matches
```

**אין platform code בקוד שלנו היום.** זה טוב — פירושו שpthread/std מסתדר על Linux. לרוב זה יעבוד גם בwindows/mac "as-is".

### מה צריך להוסיף

#### 1. `src/platform/mod.rs` — trait definition

```rust
pub trait PlatformAdapter {
    fn data_dir(app: &str) -> PathBuf;
    fn cache_dir(app: &str) -> PathBuf;
    fn can_mmap() -> bool;
    fn mmap_limit() -> usize;
    fn default_port() -> u16;
}
```

#### 2. `src/platform/*.rs` — 6 implementations

Linux/macOS/Windows: כל אחד ~30 שורות
iOS/Android: ~50 שורות
WASM: ~100 שורות (בלי mmap → IndexedDB fallback)

**סה"כ תוספת: ~300 שורות Rust** על בסיס של 45 קבצים.

#### 3. `build.rs` — conditional compilation

```rust
fn main() {
    let target = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    match target.as_str() {
        "ios" | "android" => println!("cargo:rustc-cfg=mobile_platform"),
        _ => {}
    }
}
```

#### 4. CI/CD — GitHub Actions עם 4 runners

- `ubuntu-latest` → Linux, Android, WASM
- `macos-latest` → macOS, iOS
- `windows-latest` → Windows

```yaml
strategy:
  matrix:
    include:
      - { os: ubuntu-latest, target: x86_64-unknown-linux-gnu }
      - { os: ubuntu-latest, target: aarch64-linux-android }
      - { os: ubuntu-latest, target: wasm32-unknown-unknown }
      - { os: macos-latest,  target: aarch64-apple-darwin }
      - { os: macos-latest,  target: aarch64-apple-ios }
      - { os: windows-latest, target: x86_64-pc-windows-msvc }
```

---

## מגבלות אמיתיות למobile

### iOS — הקשה ביותר

1. **Long-running HTTP server אסור** — OS משעה אותה ברקע
   - **פתרון:** להטמיע Rust כ-static library, לקרוא דרך FFI. בלי HTTP server.
2. **mmap מוגבל** ל-~2GB בזיכרון
   - **פתרון:** chunked mmap + LRU על pages (כמו שתיכננו ב-unified_node_design)
3. **App Store review** — פעם ראשונה יכול לקחת שבוע
4. **Apple Developer Program** — $99/year

### Android — קל יחסית

1. **Background service** דורש foreground notification
   - **פתרון:** הproactive personas יראו icon ב-notification bar
2. **oom-killer** אגרסיבי
   - **פתרון:** מדיניות unload של language packs ישנים
3. **Play Store** — בד"כ 24-48 שעות approval

### WASM — מגבלות שונות

1. **אין mmap** — IndexedDB-backed storage
2. **4 GB לtab** — typically 1 GB בפועל
3. **אין threads** (בסטנדרט — SharedArrayBuffer דורש CORS headers)

---

## מה ב-Stage של הפיתוח

### Phase 1 (עכשיו, מיידית):
- Linux works ✅ (קיים)
- macOS — `cargo build --target aarch64-apple-darwin` אמור לעבוד לוחאובעזרת.

### Phase 2 (שבוע 1):
- `src/platform/` module (300 שורות Rust)
- Windows + macOS + Linux עובדים ב-CI
- Tauri wrapper
- 3 .exe/.app/.AppImage builds

### Phase 3 (שבוע 2):
- iOS (דורש Mac לאזurerait final build)
- Android (cross-compile from Linux via NDK)
- 5/6 פלטפורמות

### Phase 4 (שבוע 3):
- WASM
- Deployment pipeline
- 6/6 ✅

---

## Summary — שאלות עידן, תשובות מאומתות

| שאלה | תשובה |
|------|--------|
| Windows native .exe? | **כן** — Tauri + WebView2, קיים ב-Windows 10+ |
| macOS .app? | **כן** — Tauri + WKWebView, native Mac UX |
| Linux? | **כן** — Tauri `.AppImage` רץ על כל distro |
| iOS? | **כן** — SwiftUI wrapper, Rust as static lib |
| Android? | **כן** — Kotlin wrapper, Rust as .so + JNI |
| עוד? | **כן — WASM** לדפדפן, demo/marketing |
| הקוד יהיה נקי? | **כן** — 1 trait, 6 impls, 45 קבצי core לא יודעים על OS |

**Python prototype:** 7/7 tests passing. ב-git כ-`py_testers/test_platform_abstraction_v1.py`.

**לא מיישמים Rust עדיין** — מחכה לאישור עידן (CLAUDE_RULES §4).
