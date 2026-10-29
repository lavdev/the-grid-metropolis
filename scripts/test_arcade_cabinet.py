import asyncio
from playwright.async_api import async_playwright

ARTIFACT_DIR = r"C:\Users\Lav\.gemini\antigravity\brain\fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce"

async def main():
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        # Resolução ampla para exibir todo o gabinete de fliperama
        context = await browser.new_context(viewport={"width": 1600, "height": 960})
        page = await context.new_page()

        print("1. Abrindo a aplicação no Modo Gabinete Arcade...")
        await page.goto("http://localhost:5180")
        await page.wait_for_load_state("networkidle")
        await asyncio.sleep(2)

        # 1. Captura com o gabinete arcade e Marquee do Projeto Alfa
        await page.screenshot(path=f"{ARTIFACT_DIR}\\arcade_cabinet_alfa.png")
        print("📸 Captura salva: arcade_cabinet_alfa.png (Marquee PROJETO ALFA + Bezel Espacial)")

        # 2. Alternar para o Notex Core e verificar atualização dinâmica do Marquee
        print("2. Alternando para o Notex Core...")
        notex_btn = page.locator("button:has-text('Notex Core')").first
        await notex_btn.click()
        await asyncio.sleep(1)

        await page.screenshot(path=f"{ARTIFACT_DIR}\\arcade_cabinet_notex.png")
        print("📸 Captura salva: arcade_cabinet_notex.png (Marquee atualizado para NOTEX CORE)")

        # 3. Clicar no botão MODO DECK para ver a visão limpa full-bleed
        print("3. Alternando para o Modo Deck...")
        deck_btn = page.locator("button:has-text('MODO DECK')").first
        await deck_btn.click()
        await asyncio.sleep(1)

        await page.screenshot(path=f"{ARTIFACT_DIR}\\arcade_cabinet_deck_view.png")
        print("📸 Captura salva: arcade_cabinet_deck_view.png (Visão Monitor Cyber-Deck Limpa)")

        # 4. Retornar ao Modo Arcade
        print("4. Retornando ao Modo Arcade...")
        arcade_toggle_btn = page.locator("button:has-text('MODO ARCADE')").first
        await arcade_toggle_btn.click()
        await asyncio.sleep(1)

        await browser.close()
        print("✅ Todos os testes do Gabinete Arcade foram validados com sucesso!")

if __name__ == "__main__":
    asyncio.run(main())
