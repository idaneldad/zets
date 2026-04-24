# AGI Simulation — What We're Testing

## Honest Premise
AGI is undefined. We can't prove AGI. We CAN test if mechanisms produce AGI-like behavior.

## 10 Operational Tests (each = a capability AGI should have)

T1. **Direct retrieval** — "What is a lemon?" → returns grounded description
T2. **Analogy-based inference** — "Does lime contain vitamin C?" (never told)
                                  → infers from lemon (structural analogy)
T3. **Default with exception** — "Can birds fly?" → yes USUALLY
                                 "Can this penguin fly?" → no (exception fires)
T4. **Multi-step composition** — "Why is lemonade sweet if lemon is sour?"
                                  → traces: lemon + sugar → lemonade, sugar=sweet
T5. **Context disambiguation** — "lemon" in car context vs fruit context
                                  → different senses activated
T6. **Substitution recommendation** — "No lemon, what instead?"
                                       → analogy to lime, vinegar (similar properties)
T7. **Honest ignorance** — "What is kumquat?" (not in graph)
                            → "I don't know" WITHOUT hallucinating
T8. **Self-correction** — User says "lemon is sweet" (wrong)
                           → ZETS disagrees with evidence
T9. **Explainable reasoning** — "Why does lemon help against colds?"
                                 → prints walk path: vit_C → immune_system → cold
T10. **Creative composition** — "Invent dessert: lemon + miso"
                                 → reasons from flavor profiles, generates plausible

## Honest Expectations
- Pass 8-10: "AGI-like features present"
- Pass 5-7: "Partial intelligence, missing pieces"
- Pass 0-4: "Not AGI, structure insufficient"

We will be HONEST about which tests fail and why.
