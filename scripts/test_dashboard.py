from playwright.sync_api import sync_playwright

def run():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page(viewport={'width': 1440, 'height': 850})
        page.goto('http://localhost:5180')
        page.wait_for_timeout(2500)
        
        # 1. Base capture with Projeto Alfa and Sidebar
        page.screenshot(path='C:/Users/Lav/.gemini/antigravity/brain/fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce/dashboard_terminals_base.png')
        print('Base dashboard captured')

        # 2. Click on Notex Core project in the sidebar
        page.locator('button:has-text("Notex Core")').first.click()
        page.wait_for_timeout(1000)
        page.screenshot(path='C:/Users/Lav/.gemini/antigravity/brain/fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce/dashboard_notex_terminal.png')
        print('Notex terminal captured')

        # 3. Switch back to Projeto Alfa and send a command
        page.locator('button:has-text("Projeto Alfa")').first.click()
        page.wait_for_timeout(500)
        page.locator('input[type="text"]').last.fill('Refatorar modulos e validar bridge Tokio')
        page.locator('button:has-text("Executar")').click()
        page.wait_for_timeout(1500)
        page.screenshot(path='C:/Users/Lav/.gemini/antigravity/brain/fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce/dashboard_command_streaming.png')
        print('Command streaming captured')

        browser.close()
    print('All test screenshots completed!')

if __name__ == '__main__':
    run()
