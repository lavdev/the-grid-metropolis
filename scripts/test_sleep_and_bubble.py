from playwright.sync_api import sync_playwright
import time

def run():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page(viewport={'width': 1440, 'height': 850})
        page.goto('http://localhost:5180')
        time.sleep(2)

        # 1. Dispara piada forçada para verificar posicionamento acima da cabeça
        page.evaluate("window.dispatchEvent(new CustomEvent('test-force-joke'));")
        time.sleep(1)
        page.screenshot(path='C:/Users/Lav/.gemini/antigravity/brain/fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce/spark_bubble_elevated.png')
        print('Bubble elevated captured')

        # 2. Dispara modo de sono profundo (Sleep Zzzz + Recarga de bateria)
        page.evaluate("window.dispatchEvent(new CustomEvent('test-force-sleep'));")
        time.sleep(3.5)
        page.screenshot(path='C:/Users/Lav/.gemini/antigravity/brain/fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce/spark_sleeping_mode.png')
        print('Sleeping mode captured')

        browser.close()
    print('Completed test successfully!')

if __name__ == '__main__':
    run()
