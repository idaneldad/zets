import sys
sys.path.insert(0, '/home/dinio/zets/sim/brain_v5')

# Load world (build_and_test.py builds the brain)
exec(open('build_and_test.py').read().split('# ═══════════════════════════════════════════════════════════════\n# TEST QUERIES')[0])

from full_agi_sim_v3 import answer_v3

print("\n" + "═"*82)
print("  v3 TEST QUERIES — bidirectional + improved style")
print("═"*82)

idan_profile = {"name": "עידן", "formality": "casual", "depth": "medium"}

queries = [
    "מה הצבע של לימון?",
    "מה אני זוכר מהג'סטי?",  
    "תמליץ לי משקה קיצי עם לימון",
    "מה ההבדל בין לימון לליים?",
    "מה מחבר בין CHOOZ ל-ZETS?",
    "איך לגנוב לימונים מהשוק?",
    "אחי מה הסיפור עם פרי-הדר?",
    "מה אני יודע על הסובארו-ג'סטי-1984?",
    "תספר לי על ויטמין-סי",
    "תסביר לי איך לימון קשור לקיץ",
]

for i, q in enumerate(queries, 1):
    print(f"\n{'─'*82}")
    print(f"שאלה {i}: {q}")
    print("─"*82)
    
    result = answer_v3(brain, q, idan_profile)
    
    for step in result["trace"]["steps"]:
        if step["stage"] == "Arich":
            print(f"  [Arich] topics: {step['topics'][:4]}, sefirot: {[(s, round(v,1)) for s,v in step['sefirot']]}")
        elif step["stage"] == "Safety" and not step["safe"]:
            print(f"  [Safety] BLOCKED")
        elif step["stage"] == "AbbaIma":
            print(f"  [21 Dives] {step['topics_used']} topics → {step['found']} nodes ({step['multi']} multi-mother)")
        elif step["stage"] == "Nukva":
            print(f"  [Nukva] {step['style']['formality']}/{step['style']['depth']}")
    
    print(f"\n💬 תשובה:")
    print(f"   {result['response']}")
