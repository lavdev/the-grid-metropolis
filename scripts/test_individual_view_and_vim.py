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

        # 2. Clicar no Droid Rusty na Sidebar para ativar visão individual
        print("2. Clicando em Rusty na sidebar para inspecionar contribuição individual...")
        rusty_card = page.locator("button:has-text('Rusty')").first
        await rusty_card.click()
        await asyncio.sleep(1)

        await page.screenshot(path=f"{ARTIFACT_DIR}\\individual_view_rusty.png")
        print("📸 Captura salva: individual_view_rusty.png")

        # 3. Clicar em Vanguard para ver sua auditoria e revisão
        print("3. Alternando filtro para Vanguard...")
        vanguard_btn = page.locator("button:has-text('Vanguard')").first
        await vanguard_btn.click()
        await asyncio.sleep(1)

        await page.screenshot(path=f"{ARTIFACT_DIR}\\individual_view_vanguard.png")
        print("📸 Captura salva: individual_view_vanguard.png")

        # 4. Abrir o Inspetor Vim
        print("4. Abrindo o Inspetor Vim Retrô...")
        vim_tab_btn = page.locator("button:has-text('INSPETOR VIM')").first
        await vim_tab_btn.click()
        await asyncio.sleep(1)

        await page.screenshot(path=f"{ARTIFACT_DIR}\\vim_inspector_rust_code.png")
        print("📸 Captura salva: vim_inspector_rust_code.png")

        # 5. Alternar para a aba contracts.md (documento de arquitetura do SPARK-01)
        print("5. Alternando para o arquivo contracts.md no Vim...")
        contracts_tab = page.locator("button:has-text('contracts.md')").first
        await contracts_tab.click()
        await asyncio.sleep(1)

        await page.screenshot(path=f"{ARTIFACT_DIR}\\vim_inspector_markdown.png")
        print("📸 Captura salva: vim_inspector_markdown.png")

        # 6. Sair do Vim (:q) e voltar ao Console
        print("6. Saindo do Vim (:q) para voltar ao Terminal...")
        close_vim_btn = page.locator("button:has-text('VOLTAR AO TERMINAL')").first
        await close_vim_btn.click()
        await asyncio.sleep(1)

        # 7. Limpar filtro e ver todos os droids
        print("7. Restaurando visão geral de todos os droids...")
        see_all_btn = page.locator("button:has-text('TODOS')").first
        await see_all_btn.click()
        await asyncio.sleep(1)

        await page.screenshot(path=f"{ARTIFACT_DIR}\\individual_view_cleared_all.png")
        print("📸 Captura salva: individual_view_cleared_all.png")

        await browser.close()
        print("✅ Todos os testes de Visão Individual e Inspetor Vim foram concluídos com sucesso!")

if __name__ == "__main__":
    asyncio.run(main())
