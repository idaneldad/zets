# Unified Node Design — Server + Client as Same Binary

**תאריך:** 23.04.2026
**מקור:** שאלת עידן על RAM, mmap, big pages, unified architecture, conversational memory

---

## הבעיה שזיהיתי לאחר בדיקה

### 1. מה באמת אוכל RAM בשרת

**לא ZETS.** בדיקת `/proc/meminfo`:
- MemTotal: 41 GB
- **MemAvailable: 24 GB** (זמין בפועל)
- Cached: 15.5 GB → ה-19GB של Wikipedia dumps ש-Linux kernel טוען
- Buffers: 2.2 GB → filesystem cache

**ZETS processes:** 670 MB סה"כ (multi_lang_wiki 163MB, 16 clients × 24MB, MCP + HTTP = 78MB).

**המסקנה:** הרושם של "RAM גבוה" היה בטעות. Linux חכם — יוריד cache ברגע שמשהו אחר יצטרך.

### 2. Server vs Client — **אינם אותה גרסה**

| רכיב | בינארי | טוען Rust engine? |
|------|--------|---------------------|
| `zets_http_api.py` | Python + subprocess calls | לא (רק קורא ל-binary) |
| `zets_mcp_server.py` | Python + subprocess calls | לא |
| `zets_client.py` | Python בלבד | **לא — prototype בלבד** |
| `zets_client_server` | Rust (504KB binary) | **כן — אבל לא בשימוש** |

**בעיה:** ה-16 clients שרצים עכשיו הם mock personas שקוראים JSON. **לא יש להם mmap, atoms, hash_registry, inference**. זו הסיבה שכל התשובות שלהם מוגבלות.

---

## העיצוב המוצע (Python prototype הוכיח — tests/test_unified_node_v1.py)

### רכיב מרכזי: `ZetsNode`

**קוד אחד, בינארי אחד** — `zets_node`. מקבל config:

```rust
struct NodeConfig {
    name: String,
    role: NodeRole,          // Server | Persona
    port: u16,
    parent_url: Option<String>,  // None for server, Some for personas
    init_graph_path: PathBuf,
    hot_page_threshold: u32,     // accesses before MADV_HUGEPAGE
    sync_interval: Duration,     // personas sync to server every N
}

enum NodeRole { Server, Persona }
```

**השימוש:**
```bash
# Server
zets_node --role=server --port=3147 --graph=data/baseline/wiki_all_domains_v1.atoms

# Persona: Idan
zets_node --role=persona --port=3251 --parent=http://localhost:3147 \
          --graph=data/personas/idan.atoms

# Persona: Yam (child)
zets_node --role=persona --port=3265 --parent=http://localhost:3147 \
          --graph=data/personas/yam.atoms --register-max=4  # no slang
```

---

## Page activity + big pages (תשובה ישירה)

### הבנה נכונה: Linux יש 3 גדלי עמודים

- **4 KB** — standard page (default)
- **2 MB** — huge page (512× גדול יותר)
- **1 GB** — gigantic page (262,144× גדול יותר)

### מה ZETS יעשה

**Adaptive page promotion** לפי access frequency (Python prototype הוכיח את הלוגיקה):

```rust
impl PageTracker {
    fn touch(&mut self, atom_id: AtomId) {
        let page_id = atom_id / ATOMS_PER_PAGE;
        self.access_counts[page_id] += 1;
        let count = self.access_counts[page_id];

        match self.page_type[page_id] {
            Small if count >= self.hot_threshold => {
                // Ask kernel to use 2MB huge page
                unsafe {
                    libc::madvise(addr, len, libc::MADV_HUGEPAGE);
                }
                self.page_type[page_id] = Huge;
            }
            Huge if count >= self.hot_threshold * 10 => {
                // Request 1GB gigantic page (if OS supports)
                self.page_type[page_id] = Giga;
            }
            _ => {}
        }
    }

    fn cool_down(&mut self) {
        // Called every 5 min — demote cold pages
        for (page_id, count) in self.access_counts.iter_mut() {
            if *count == 0 {
                if self.page_type[*page_id] != Small {
                    unsafe {
                        libc::madvise(addr, len, libc::MADV_COLD);
                    }
                    self.page_type[*page_id] = Small;
                }
            }
            *count = count.saturating_sub(1);
        }
    }
}
```

**תוצאה:** 
- קונספטים חמים (שכל 10 ה-personas שואלים עליהם) → 2MB page → **אחוז TLB miss נופל**
- קונספטים קרים → 4KB → OS יכול לשחרר בחופשיות

---

## Conversational memory (חדש — הצורך שעידן זיהה)

### הבעיה

היום: המשתמש שואל "ספר לי על Python". ה-bot עונה. המשתמש אומר "כן" או "3". **ה-bot לא יודע אל מה זה מתייחס.**

### הפתרון ב-graph

כל turn של שיחה נשמר כ-atoms + edges:

```
atom A1 (UserInput): "ספר לי על Python"
atom A2 (AssistantReply): "I know about Python"
atom Q1 (PendingQuestion): "Would you like to see code?"
atom Q2 (PendingQuestion): "Compare with another language?"
atom Q3 (PendingQuestion): "Strengths and weaknesses?"

edge A1 -replied_by-> A2
edge A2 -has_followup-> Q1
edge A2 -has_followup-> Q2
edge A2 -has_followup-> Q3

[ConversationTurn turn_id=0, pending_questions=[Q1, Q2, Q3]]
```

**כשהמשתמש עונה:**
- "yes" / "כן" → הsystem מחפש pending_by_turn של 5 turns אחרונים, לוקח את הראשון
- "3" → אותו תהליך, אבל לוקח את האינדקס ה-3
- "the one about comparing" → word-match על q.content

**Python prototype הוכיח** — `_resolve_context()` שופט בין 3 האסטרטגיות.

---

## Sync + PII + בטיחות

### Sync protocol (persona → server)

כל 5 דקות, persona שולח ל-server:
1. **Hash digest** של ה-atoms שנוספו (לא התוכן)
2. **Gap request**: "אלה hashes ש-I have, מה חסר לי?"
3. Server משיב עם atoms + edges שחסרים

**לא נשלח:** שיחות מלאות, queries של user, PII.

### Privacy by design

- `parent_url` הוא **הדבר היחיד** שmakes a node a persona
- Personas יודעים מה לא לשלוח — conversations עם kind=UserInput **נשארים מקומיים**
- Server לא יודע על המשתמש שמדבר עם Yam

---

## מה לעשות בתור הבא

### מה עובד (Python prototype):
✅ Unified node class  
✅ Same code, different config  
✅ Conversation stored in graph  
✅ yes/no/number resolution  
✅ Page activity tracking  
✅ Cold-page demotion logic  

### Rust implementation (הצעד הבא — רק אחרי אישור עידן):
1. Extend `ZetsEngine` with `NodeRole` enum
2. Add `PageTracker` using `libc::madvise`
3. Add `ConversationAtom` variant to `AtomKind`
4. Add `conversation` module — turn storage + context resolution
5. CLI: `src/bin/zets_node.rs` that handles both roles
6. **Retire Python `zets_client.py`** — replace with Rust binary

### טסטים שצריך לעבור לפני שהRust נפרס:
- [ ] Server startup: < 5 sec with 14M articles
- [ ] Persona startup: < 1 sec with 1000 atoms
- [ ] Conversation: "yes" resolves within 100ms
- [ ] 16 personas × 50 MB RAM max (currently 24 MB in Python — Rust should be lower)
- [ ] Page promotion: verified with `perf stat` (TLB hits / misses)

---

## Summary

| שאלה שלך | תשובה | הוכחה |
|-----------|--------|--------|
| RAM גבוה? | לא — זה cache. ZETS עצמו 670MB | `/proc/meminfo`, ps rss |
| Server = Client? | **עכשיו לא**. מוצע: `ZetsNode` אחד | `test_unified_node_v1.py [1]` |
| Big page adaptive? | כן — MADV_HUGEPAGE / MADV_COLD | prototype `PageTracker` |
| כל שיחה בגרף? | כן — כ-atoms+edges | prototype `ConversationTurn` |
| yes/3 מוצא context? | כן — pending_by_turn index | `test_unified_node_v1.py [3,4]` |

**Python prototype הוכיח שההצעה ההנדסית עובדת.** מוכן להתחיל Rust implementation ברגע שעידן יאשר.
