from playwright.sync_api import sync_playwright
import time

def run():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page(viewport={'width': 1440, 'height': 850})
        page.goto('http://localhost:5180')
        time.sleep(2)

        # 1. Posição inicial do terminal com os 3 droids e o stepper
        page.screenshot(path='C:/Users/Lav/.gemini/antigravity/brain/fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce/pipeline_step0_ready.png')
        print('Step 0 captured')

        # 2. Digitar instrução no terminal para disparar a esteira
        page.locator('input[type="text"]').last.fill('Implementar driver de persistencia assincrona com SQLite WASM')
        page.locator('button:has-text("Executar")').click()

        # 3. Etapa 1: Arquiteto (SPARK-01) gerando especificação
        time.sleep(1.8)
        page.screenshot(path='C:/Users/Lav/.gemini/antigravity/brain/fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce/pipeline_step1_architect.png')
        print('Step 1 (Architect) captured')

        # 4. Etapa 2: Handoff para o Coder (Rusty) codificando
        time.sleep(2.8)
        page.screenshot(path='C:/Users/Lav/.gemini/antigravity/brain/fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce/pipeline_step2_coder.png')
        print('Step 2 (Coder) captured')

        # 5. Etapa 3: Handoff para o Revisor (Vanguard) e conclusão
        time.sleep(3.6)
        page.screenshot(path='C:/Users/Lav/.gemini/antigravity/brain/fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce/pipeline_step3_reviewed.png')
        print('Step 3 (Reviewed & Approved) captured')

        browser.close()
    print('All pipeline test screenshots completed!')

if __name__ == '__main__':
    run()
