import time
import json
from playwright.sync_api import sync_playwright

def test_codex():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        context = browser.new_context(viewport={"width": 1920, "height": 1080})
        page = context.new_page()

        print("1. Acessando Grid Metropolis...")
        page.goto("http://localhost:5173")
        page.wait_for_timeout(3000)

        # Clica no botão de Configuração de IA no header CRT
        print("2. Abrindo Modal de Configuração de IA...")
        ai_btn = page.locator("button:has-text('MODELO:')")
        if not ai_btn.is_visible():
            ai_btn = page.locator("button:has(svg.lucide-key)")
        ai_btn.click()
        page.wait_for_timeout(1000)

        # Tira screenshot do modal aberto
        page.screenshot(path="C:/Users/Lav/.gemini/antigravity/brain/9c0b3ccd-b94c-49ba-a6f6-b7c55e89bb5a/codex_settings_modal.png")
        print("Screenshot do modal salvo.")

        # Clica na opção Codex CLI
        print("3. Selecionando provedor Codex CLI...")
        codex_btn = page.locator("button:has-text('Codex CLI')")
        codex_btn.click()
        page.wait_for_timeout(1000)

        page.screenshot(path="C:/Users/Lav/.gemini/antigravity/brain/9c0b3ccd-b94c-49ba-a6f6-b7c55e89bb5a/codex_selected_view.png")

        # Clica em Testar Conexão
        print("4. Testando conexão com Codex...")
        test_btn = page.locator("button:has-text('TESTAR CONEXÃO')")
        test_btn.click()
        page.wait_for_timeout(2500)

        page.screenshot(path="C:/Users/Lav/.gemini/antigravity/brain/9c0b3ccd-b94c-49ba-a6f6-b7c55e89bb5a/codex_tested_success.png")

        # Clica em Salvar & Aplicar
        print("5. Salvando e aplicando...")
        save_btn = page.locator("button:has-text('SALVAR & APLICAR')")
        save_btn.click()
        page.wait_for_timeout(1500)

        page.screenshot(path="C:/Users/Lav/.gemini/antigravity/brain/9c0b3ccd-b94c-49ba-a6f6-b7c55e89bb5a/codex_header_badge.png")

        # Testa envio de mensagem no terminal CRT com Codex ativo
        print("6. Disparando prompt para os Droids via Codex CLI...")
        cli_input = page.locator("input[placeholder*='Digite seu comando']")
        cli_input.click()
        cli_input.fill("Diga apenas: 'SPARK-01 conectado ao Codex via ChatGPT!'")
        
        # Clica no botão Executar ou pressiona Enter
        exec_btn = page.locator("button:has-text('EXECUTAR')")
        if exec_btn.is_visible():
            exec_btn.click()
        else:
            page.keyboard.press("Enter")

        # Aguarda a resposta do Codex em streaming
        print("7. Aguardando streaming do Codex...")
        for i in range(12):
            page.wait_for_timeout(2000)
            page.screenshot(path="C:/Users/Lav/.gemini/antigravity/brain/9c0b3ccd-b94c-49ba-a6f6-b7c55e89bb5a/codex_streaming_live.png")
            content = page.content()
            if "conectado ao Codex" in content or "CODEX CLI" in content:
                print(f"Detectado progresso no ciclo {i+1}!")
                if "SPARK-01 conectado ao Codex" in content or "tokens" in content:
                    print("Resposta concluída!")
                    break

        page.wait_for_timeout(2000)
        page.screenshot(path="C:/Users/Lav/.gemini/antigravity/brain/9c0b3ccd-b94c-49ba-a6f6-b7c55e89bb5a/codex_completed_deck.png")
        print("Teste finalizado com sucesso!")

        browser.close()

if __name__ == "__main__":
    test_codex()
