import time
from playwright.sync_api import sync_playwright

def test_full_pipeline():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page(viewport={"width": 1920, "height": 1080})

        print("1. Carregando The Grid Metropolis...")
        page.goto("http://localhost:5173")
        page.wait_for_timeout(3000)

        # Dispara comando no terminal CRT
        print("2. Enviando instrução para a esteira multi-droid...")
        cli_input = page.locator("input[placeholder*='Digite seu comando']")
        cli_input.fill("Crie uma função simples de soma em Rust com teste unitário.")
        
        exec_btn = page.locator("button:has-text('EXECUTAR')")
        exec_btn.click()

        print("3. Monitorando esteira em tempo real...")
        for second in range(30):
            page.wait_for_timeout(1000)
            content = page.content()
            if "ESTEIRA CONCLUÍDA" in content or "100%" in content or "fn soma" in content or "VANGUARD" in content:
                print(f"Progresso detectado em {second+1}s!")
                if "100%" in content or "ESTEIRA CONCLUÍDA" in content:
                    print("Esteira multi-droid totalmente concluída!")
                    break

        page.screenshot(path="C:/Users/Lav/.gemini/antigravity/brain/9c0b3ccd-b94c-49ba-a6f6-b7c55e89bb5a/codex_pipeline_full_run.png")
        print("Screenshot final salvo em codex_pipeline_full_run.png")
        browser.close()

if __name__ == "__main__":
    test_full_pipeline()
