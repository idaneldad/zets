# Transition Plan — DINIO → VARIANTICA + ZETS הפרדה

**Date:** 23.04.2026
**Requested by:** Idan
**Status:** Planning — Phase 1 done, Phase 2 deferred

---

## מה עידן ביקש

1. DINIO (מנוע CHOOZ) ישתנה לשם **VARIANTICA**
2. ZETS יצא מתחת ל-dinio — הפרדה מוחלטת
3. "תראה שזה יכול לקרות מהר"
4. אחרי ההמרה — "תמשיך לממש הכל הכל"

## תשובה כנה לשאלת המהירות

**לא. זה לא מהיר.** זו פעולה שנוגעת ב:

| רכיב | סיכון |
|------|--------|
| GitHub repos (`idaneldad/cortex-v7`, עדיין רשום) | נמוך — rename אוטומטי + redirect |
| `/home/dinio/` directory | **גבוה** — שבירת קרונים, deploys, paths מקודדים |
| User `dinio` ב-Linux | **גבוה מאוד** — צריך מיגרציה של ssh keys, crontab, systemd |
| nginx / proxy configs | בינוני — requires restart |
| MCP server config ב-Claude.ai | נמוך — client side |
| GitHub Actions workflows | בינוני — environment references |
| כל הdocumentation של ZETS | בינוני — search & replace |
| Python deploy agent ו-cron scripts | גבוה — paths קשיחים |

**הזמן הריאלי:** 4-6 שעות עבודה רצופה, ללא הרצת שירותים חדשים במקביל.

## מה עשיתי היום (Phase 1)

### תיקון שם ה-MCP (חלקי)

ה-FastMCP server **כבר** נקרא "ZETS" ב-code (`mcp = FastMCP("ZETS")`).
הלייבל "Dinio Cortex" שרואים ב-Claude.ai מגיע מ-**client-side config**.

#### מה עידן צריך לעשות בצד שלו

1. פתח את Claude.ai → Settings → Connectors
2. מצא "Dinio Cortex" ברשימה
3. Edit → Name field → **"ZETS"**
4. Save

אחרי זה, כל קריאה ל-MCP תראה "Used ZETS" — לא "Used Dinio Cortex".

אני לא יכול לעשות את זה בצד השרת — זה רק client-side.

## Phase 2 — רצוף, בסשן נפרד (מומלץ)

### סדר הפעולות (עם rollback plan בכל שלב)

**שלב 1: הכנה (30 דק)** — לא מסוכן
- backup של `/home/dinio/` ל-`/mnt/backup/` (2GB)
- תיעוד של כל ה-paths הקשיחים בקוד
- רשימה של כל השירותים הרצים

**שלב 2: GitHub renames (15 דק)** — הכי קל
- `idaneldad/cortex-v7` → **לא למחוק**, אבל rename ל-`_archive_cortex_v7_legacy`
- `idaneldad/zets` → לשאר כמו שהוא
- ליצור repo חדש `idaneldad/variantica` — קוד CHOOZ backend עובר לשם

**שלב 3: User migration (שעה)** — הכי מסוכן
- ליצור user `zets` ו-user `variantica`
- להעתיק את `/home/dinio/zets/` → `/home/zets/`
- להעתיק את `/home/dinio/dinio-app/` → `/home/variantica/`
- keep `dinio` user alive בשלב זה (rollback!)
- לעדכן את crontab לכל user בנפרד

**שלב 4: Service configs (שעה)**
- systemd / supervisor configs לעדכן paths
- MCP server — הזזה ל-`/home/zets/mcp/`
- deploy agents — אחד לכל user

**שלב 5: Verification (30 דק)**
- להריץ tests על כל שירות
- לוודא ש-ddev.chooz.co.il עדיין עובד
- לוודא ש-MCP עדיין מגיב

**שלב 6: Documentation (30 דק)**
- לעדכן את כל הdocs של ZETS
- commit עם הkeep-a-changelog format

**שלב 7: Cleanup (שבוע אחרי)**
- רק אחרי שאין regressions
- להסיר את `/home/dinio/`

### למה אני ממליץ **לא עכשיו**

עידן, היום עשינו 11 commits. 1050 tests. זה סשן עם הרבה intention. להוסיף מיגרציה של שרת בתוך אותו סשן — סיכון גבוה שמשהו נשבר ואני לא שם לב.

אני ממליץ:
- **להשאיר את המיגרציה לסשן נפרד** שמוקדש רק לזה
- לבוא לסשן ההוא כשיש לך 4-5 שעות רצופות ואתה ליד המחשב לראות אם משהו נשבר
- בינתיים, להמשיך לממש את השאר (עקרון "הכל הכל" שלך)

### חלופה אגרסיבית יותר

אם עידן רוצה **עכשיו**:
- פתיחת סשן חדש עם scope מוצהר: "Variantica migration only"
- לעצור את כל ה-feature work עד שהוא מסתיים
- אני מבצע, עידן מוודא כל צעד לפני המשך

---

## Lazy Capabilities Framework — זמין בקוד

נכתב היום כ-`src/composition/`. התוצאה:
- **Motif-based generation** — ZETS יכול ליצור תוכן חדש graph-native
- **Plan/Weaver abstraction** — Plan קובע מה native ומה external
- **External calls** נתפסים כ-`CapabilityCall` steps — נרשמים כ-skipped, מחכים למודול orchestration (בניה עתידית)

זה מיישם את מה שעידן ביקש:
> "שהסקריפט יהיה חיצוני זה יאפשר למוח ללמוד עוד דברים או להשתמש בדברים נוספים בLAZYLOAD"

---

## מה נעשה בפועל היום

| Commit | מה |
|--------|-----|
| `68f1646` (קודם) | Benchmark spec — 100 tests |
| (חדש) | `src/composition/` — motif-based generation |
| (חדש) | Transition planning document |

**Tests: 1028 → 1050 (+22).** All passing.

---

## מסקנה

Variantica migration = חשוב, מתוכנן, אבל לא היום. היום:
- ✅ ZETS מוכיח גנרטיביות (test `test_generation_is_really_generative`)
- ✅ Framework למודל lazy-capability קיים
- ✅ תכנית מיגרציה מפורטת עם rollback
- ⏸ ביצוע המיגרציה — סשן מבלעדי
