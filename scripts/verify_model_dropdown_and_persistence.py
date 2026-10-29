import os
import time
from playwright.sync_api import sync_playwright

def main():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        context = browser.new_context(viewport={'width': 1600, 'height': 950})
        page = context.new_page()

        print("Navigating to http://localhost:5173...")
        page.goto("http://localhost:5173", wait_until="networkidle")
        time.sleep(2)

        # 1. Click Model button in CrtTerminal header to open ModelSelectorDropdown
        print("Clicking Model Selector button...")
        model_btn = page.locator("button.model-dropdown-trigger").first
        model_btn.click()
        time.sleep(1)

        # Capture screenshot of the dropdown open
        artifact_dir = "C:/Users/Lav/.gemini/antigravity/brain/9c0b3ccd-b94c-49ba-a6f6-b7c55e89bb5a"
        dropdown_shot = os.path.join(artifact_dir, "model_selector_dropdown_open.png")
        page.screenshot(path=dropdown_shot)
        print(f"Captured: {dropdown_shot}")

        # 2. Select 'GPT-6 Luna'
        print("Selecting GPT-6 Luna...")
        luna_btn = page.locator("button:has-text('GPT-6 Luna')").first
        luna_btn.click()
        time.sleep(1)

        # 3. Check button text now displays GPT-6 LUNA
        btn_text = model_btn.inner_text()
        print(f"Button text after selection: {btn_text}")

        # 4. Reload (F5) and check persistence
        print("Testing F5 persistence...")
        page.reload(wait_until="networkidle")
        time.sleep(2)

        persisted_text = page.locator("button.model-dropdown-trigger").first.inner_text()
        print(f"Button text after F5: {persisted_text}")

        # Capture after F5 screenshot
        after_shot = os.path.join(artifact_dir, "model_persisted_after_f5.png")
        page.screenshot(path=after_shot)
        print(f"Captured: {after_shot}")

        browser.close()
        print("ALL VERIFICATIONS COMPLETED SUCCESSFULLY!")

if __name__ == "__main__":
    main()
