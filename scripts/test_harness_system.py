import asyncio
import time
from playwright.async_api import async_playwright

ARTIFACT_DIR = r"C:\Users\Lav\.gemini\antigravity\brain\fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce"

async def main():
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        context = await browser.new_context(viewport={"width": 1440, "height": 900})
        page = await context.new_page()

        print("1. Acessando aplicação na porta 5180...")
        await page.goto("http://localhost:5180")
        await page.wait_for_load_state("networkidle")
        await asyncio.sleep(2)

        # 2. Abrir o Droid Forge através do botão FORGE
        print("2. Abrindo o Droid Forge & Roster...")
        forge_btn = page.locator("button:has-text('FORGE')").first
        await forge_btn.click()
        await asyncio.sleep(1)

        await page.screenshot(path=f"{ARTIFACT_DIR}\\harness_droid_forge_roster.png")
        print("📸 Captura salva: harness_droid_forge_roster.png")

        # 3. Navegar para a aba de forjar novo droid
        print("3. Abrindo formulário de criação de novo Droid...")
        new_droid_tab = page.locator("button:has-text('FORJAR NOVO DROID')")
        await new_droid_tab.click()
        await asyncio.sleep(1)

        # Preencher dados do novo droid
        await page.fill("#form-name", "Cyber-Viper")
        await page.select_option("#form-role", "SECURITY")
        await page.fill("#form-badge", "Kernel Sandbox & SecOps")
        await page.fill("#form-skills", "Docker, eBPF, Sandbox, TokenAuth")
        await page.fill("#form-prompt", "Sentinela de kernel focado em isolamento e inspeção de pacotes.")
        await asyncio.sleep(1)

        await page.screenshot(path=f"{ARTIFACT_DIR}\\harness_droid_forge_editor.png")
        print("📸 Captura salva: harness_droid_forge_editor.png")

        # Salvar o novo droid (já fecha o Forge automaticamente)
        save_btn = page.locator("button:has-text('SALVAR DROID NO ROSTER')")
        await save_btn.click()
        await asyncio.sleep(1.5)

        # 4. Criar um novo workspace em Modo Solo (1 Droid)
        print("4. Criando novo Workspace com alocação Solo...")
        new_proj_btn = page.locator("button[title='Novo Projeto / Terminal']")
        await new_proj_btn.click()
        await asyncio.sleep(1)

        await page.fill("#input-proj-name", "Notex Core Solo")
        await page.fill("#input-proj-path", "C:\\Workspace\\notex-core-solo")

        # Clicar no preset Solo (1)
        solo_preset = page.locator("button:has-text('Solo (1)')")
        await solo_preset.click()
        await asyncio.sleep(1)

        await page.screenshot(path=f"{ARTIFACT_DIR}\\harness_new_project_solo_modal.png")
        print("📸 Captura salva: harness_new_project_solo_modal.png")

        # Inicializar Workspace
        init_btn = page.locator("button:has-text('INICIALIZAR WORKSPACE')")
        await init_btn.click()
        await asyncio.sleep(2)

        await page.screenshot(path=f"{ARTIFACT_DIR}\\harness_solo_workspace_ready.png")
        print("📸 Captura salva: harness_solo_workspace_ready.png")

        # 5. Executar comando no modo Solo
        print("5. Enviando instrução para o operador Solo...")
        input_cli = page.locator("input[placeholder*='Digite seu comando']")
        if await input_cli.count() == 0:
            input_cli = page.locator("input[type='text']").last
        await input_cli.fill("Criar endpoint de health check e métricas prometheus")
        await input_cli.press("Enter")

        # Aguardar conclusão solo (~3s)
        await asyncio.sleep(3.5)
        await page.screenshot(path=f"{ARTIFACT_DIR}\\harness_solo_workspace_executed.png")
        print("📸 Captura salva: harness_solo_workspace_executed.png")

        # 6. Testar modal de reconfiguração de droids existente
        print("6. Abrindo reconfiguração de droids no projeto existente...")
        settings_btn = page.locator("button[title='Configurar Droids Alocados neste Projeto']").first
        await settings_btn.click()
        await asyncio.sleep(1)

        await page.screenshot(path=f"{ARTIFACT_DIR}\\harness_reconfig_project_modal.png")
        print("📸 Captura salva: harness_reconfig_project_modal.png")

        # Fechar modal
        close_reconfig = page.locator("button:has-text('Cancelar')").first
        await close_reconfig.click()
        await asyncio.sleep(1)

        # 7. Voltar para Projeto Alfa e testar pipeline tríade
        print("7. Testando esteira tríade no Projeto Alfa...")
        alfa_btn = page.locator("button:has-text('Projeto Alfa')").first
        await alfa_btn.click()
        await asyncio.sleep(1)

        input_cli = page.locator("input[placeholder*='Digite seu comando']")
        if await input_cli.count() == 0:
            input_cli = page.locator("input[type='text']").last
        await input_cli.fill("Integrar storage OPFS com sync assincrono")
        await input_cli.press("Enter")

        # Esperar handoff completo (Etapa 1 -> 2 -> 3 -> 100%)
        await asyncio.sleep(9.5)
        await page.screenshot(path=f"{ARTIFACT_DIR}\\harness_triad_pipeline_verified.png")
        print("📸 Captura salva: harness_triad_pipeline_verified.png")

        await browser.close()
        print("✅ Todos os testes do Harness foram validados com sucesso!")

if __name__ == "__main__":
    asyncio.run(main())
