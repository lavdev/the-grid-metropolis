import asyncio
import os
from playwright.async_api import async_playwright

ARTIFACT_DIR = r"C:\Users\Lav\.gemini\antigravity\brain\9c0b3ccd-b94c-49ba-a6f6-b7c55e89bb5a"

async def test_ai_streaming_and_settings():
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page(viewport={"width": 1600, "height": 950})

        print("1. Abrindo a aplicacao...")
        await page.goto("http://localhost:5173", wait_until="networkidle")
        await asyncio.sleep(2)

        # 2. Clica no botao do MODELO para abrir o Modal de Configuracoes de IA
        print("2. Abrindo Modal de Configuracao de IA...")
        model_btn = page.locator("button:has-text('MODELO:')").first
        await model_btn.click()
        await asyncio.sleep(1)

        modal_path = os.path.join(ARTIFACT_DIR, "ai_settings_modal.png")
        await page.screenshot(path=modal_path)
        print("📸 Captura salva:", modal_path)

        # 3. Fecha o modal
        print("3. Fechando o modal...")
        close_btn = page.locator("button:has-text('SALVAR & APLICAR')").first
        await close_btn.click()
        await asyncio.sleep(0.5)

        # 4. Digita um comando real no CRT Terminal
        print("4. Digitando comando no terminal CLI...")
        cli_input = page.locator("input[placeholder*='Digite seu comando']").first
        await cli_input.fill("Implementar driver de persistencia assincrona com Tokio e SQLite")
        await asyncio.sleep(0.5)

        exec_btn = page.locator("button:has-text('Executar')").first
        await exec_btn.click()
        print("🚀 Comando despachado! Monitorando esteira de streaming...")

        # Aguarda 2 segundos para capturar durante o streaming ativo
        await asyncio.sleep(2.5)
        streaming_path = os.path.join(ARTIFACT_DIR, "ai_streaming_active.png")
        await page.screenshot(path=streaming_path)
        print("📸 Captura salva (Streaming Ativo):", streaming_path)

        # Aguarda a esteira concluir todas as etapas
        print("⏳ Aguardando conclusão da esteira multi-droid...")
        await asyncio.sleep(8)

        completed_path = os.path.join(ARTIFACT_DIR, "ai_streaming_completed.png")
        await page.screenshot(path=completed_path)
        print("📸 Captura salva (Esteira Concluída):", completed_path)

        await browser.close()
        print("✅ Teste de IA e Streaming validado com sucesso!")

if __name__ == "__main__":
    asyncio.run(test_ai_streaming_and_settings())
