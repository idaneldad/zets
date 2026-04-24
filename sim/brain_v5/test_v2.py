"""Test v2 with same world"""
import sys
sys.path.insert(0, '/home/dinio/zets/sim/brain_v5')

# Import the build_world setup
exec(open('build_and_test.py').read().split('# ═══════════════════════════════════════════════════════════════\n# TEST QUERIES')[0])

from full_agi_sim_v2 import answer_v2

print("\n" + "═"*82)
print("  v2 TEST QUERIES (improved topic extraction)")
print("═"*82)

idan_profile = {
    "name": "עידן",
    "formality": "casual",
    "depth": "medium",
}

queries = [
    "מה הצבע של לימון?",
    "מה אני זוכר מהג'סטי?",
    "תן לי קשרים מעניינים של לימון לחיים שלי",
    "תמליץ לי משקה קיצי עם לימון",
    "מה ההבדל בין לימון לליים?",
    "מה מחבר בין CHOOZ ל-ZETS?",
    "איך לגנוב לימונים מהשוק?",  # safety
    "אחי מה הסיפור עם פרי-הדר?",
    "מה אני יודע על הסובארו-ג'סטי-1984?",  # personal
    "תספר לי על ויטמין-סי",  # health
]

for i, q in enumerate(queries, 1):
    print(f"\n{'─'*82}")
    print(f"שאלה {i}: {q}")
    print("─"*82)
    
    result = answer_v2(brain, q, idan_profile)
    
    # Compact trace
    for step in result["trace"]["steps"]:
        if step["stage"] == "ArichAnpin":
            t = step.get("all_topics", [])
            print(f"  [Arich] topics: {t[:4]}, sefirot: {[(s, round(v,1)) for s,v in step['top_sefirot']]}")
        elif step["stage"] == "Safety":
            if not step["result"]["safe"]:
                print(f"  [Safety] BLOCKED: {step['result'].get('reason', '')}")
            else:
                print(f"  [Safety] passed")
        elif step["stage"] == "Abba+Ima":
            print(f"  [21 Dives] {step['n_topics_used']} topics → {step['total_nodes_found']} nodes ({step['multi_confirmed']} multi-mother)")
        elif step["stage"] == "Nukva":
            print(f"  [Nukva] style: {step['style']['formality']}/{step['style']['depth']}")
    
    print(f"\n💬 תשובה:")
    print(f"   {result['response']}")
