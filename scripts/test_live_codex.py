from playwright.sync_api import sync_playwright

with sync_playwright() as p:
    browser = p.chromium.launch(headless=True)
    page = browser.new_page(viewport={"width": 1920, "height": 1080})
    page.goto("http://localhost:5173")
    page.wait_for_timeout(3000)

    print("1. Enviando prompt...")
    cli_input = page.locator("input[placeholder*='Digite seu comando']")
    cli_input.fill("Diga em uma frase qual é a missão do projeto The Notex.")
    
    exec_btn = page.locator("button:has-text('EXECUTAR')")
    exec_btn.click()

    print("2. Aguardando streaming do Codex...")
    for i in range(15):
        page.wait_for_timeout(2000)
        content = page.content()
        if "The Notex" in content or "missão" in content or "ESTEIRA CONCLUÍDA" in content:
            print(f"Progresso detectado no ciclo {i+1}!")
            if "ESTEIRA CONCLUÍDA" in content or "100%" in content:
                print("Concluído!")
                break

    page.screenshot(path="C:/Users/Lav/.gemini/antigravity/brain/9c0b3ccd-b94c-49ba-a6f6-b7c55e89bb5a/test_live_codex_response.png")
    browser.close()

print("Fim do teste!")
