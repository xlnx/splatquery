import os
import json
import requests

ROOT_DIR = os.path.dirname(__file__)
I18N_RC_DIR = os.path.join(ROOT_DIR, "resources/assets/i18n")
RUST_LOCALES_DIR = os.path.join(ROOT_DIR, "locales")

locales = [
    os.path.splitext(s)[0] for s in os.listdir(I18N_RC_DIR) if s.endswith(".json")
]

for locale in locales:
    splatnet = requests.get(f"https://splatoon3.ink/data/locale/{locale}.json").json()
    file_name = f"{locale}.json"
    with open(os.path.join(I18N_RC_DIR, "../splatnet", file_name), "w", encoding='utf8') as f:
        json.dump(splatnet, f, ensure_ascii=False)
    with open(os.path.join(I18N_RC_DIR, file_name), "r", encoding='utf8') as f:
        i18n = json.load(f)
    i18n.update({"splatnet": splatnet})
    with open(os.path.join(RUST_LOCALES_DIR, file_name), "w", encoding='utf8') as f:
        json.dump(i18n, f, ensure_ascii=False)
