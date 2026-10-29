import asyncio
from playwright.async_api import async_playwright

ARTIFACT_DIR = r"C:\Users\Lav\.gemini\antigravity\brain\fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce"

async def main():
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        context = await browser.new_context(viewport={"width": 1440, "height": 900})
        page = await context.new_page()

        print("1. Abrindo a aplicação...")
        await page.goto("http://localhost:5180")
        await page.wait_for_load_state("networkidle")
        await asyncio.sleep(2)

        # 1. Captura do estado inicial limpo (Header sem botões redundantes)
        await page.screenshot(path=f"{ARTIFACT_DIR}\\sidebar_glow_0_idle_clean_header.png")
        print("📸 Captura salva: sidebar_glow_0_idle_clean_header.png")

        # 2. Disparar instrução no Projeto Alfa para iniciar a esteira
        input_cli = page.locator("input[placeholder*='Digite seu comando']")
        await input_cli.fill("Refatorar módulo de criptografia e auditoria")
        await input_cli.press("Enter")
        print("2. Comando enviado! Acompanhando esteira...")

        # Aguardar ~1s para Etapa 1 (Arquiteto: SPARK-01 ativo)
        await asyncio.sleep(1.2)
        await page.screenshot(path=f"{ARTIFACT_DIR}\\sidebar_glow_1_architect_spark.png")
        print("📸 Captura salva: sidebar_glow_1_architect_spark.png (SPARK-01 em Ciano)")

        # Aguardar ~2.2s para transição para Etapa 2 (Coder: Rusty ativo)
        await asyncio.sleep(2.4)
        await page.screenshot(path=f"{ARTIFACT_DIR}\\sidebar_glow_2_coder_rusty.png")
        print("📸 Captura salva: sidebar_glow_2_coder_rusty.png (Rusty em Verde Néon)")

        # Aguardar ~3.2s para transição para Etapa 3 (Revisor: Vanguard ativo)
        await asyncio.sleep(3.2)
        await page.screenshot(path=f"{ARTIFACT_DIR}\\sidebar_glow_3_reviewer_vanguard.png")
        print("📸 Captura salva: sidebar_glow_3_reviewer_vanguard.png (Vanguard em Rosa Choque)")

        # Aguardar conclusão da esteira (~3s)
        await asyncio.sleep(3.5)
        await page.screenshot(path=f"{ARTIFACT_DIR}\\sidebar_glow_4_pipeline_done.png")
        print("📸 Captura salva: sidebar_glow_4_pipeline_done.png (100% Concluído)")

        await browser.close()
        print("✅ Sequência de capturas de foco néon concluída com sucesso!")

if __name__ == "__main__":
    asyncio.run(main())
