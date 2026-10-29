from playwright.sync_api import sync_playwright

with sync_playwright() as p:
    browser = p.chromium.launch(headless=True)
    page = browser.new_page(viewport={"width": 1920, "height": 1080})
    page.goto("http://localhost:5173")
    page.wait_for_timeout(3000)
    page.screenshot(path="C:/Users/Lav/.gemini/antigravity/brain/9c0b3ccd-b94c-49ba-a6f6-b7c55e89bb5a/terminal_refined_fonts_1080p.png")

    cli_input = page.locator("input[placeholder*='Digite seu comando']")
    cli_input.fill("Status operacional do projeto The Notex: verificar integridade da esteira.")
    page.wait_for_timeout(500)
    page.screenshot(path="C:/Users/Lav/.gemini/antigravity/brain/9c0b3ccd-b94c-49ba-a6f6-b7c55e89bb5a/terminal_input_refined_fonts.png")

    browser.close()

print("Screenshots captured successfully!")
