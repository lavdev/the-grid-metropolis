from playwright.sync_api import sync_playwright
import time

def run():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page(viewport={'width': 1440, 'height': 850})
        page.goto('http://localhost:5180')
        
        # 1. Posição inicial
        time.sleep(2)
        page.screenshot(path='C:/Users/Lav/.gemini/antigravity/brain/fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce/spark_flight_1.png')
        print('Flight 1 captured')
        
        # 2. Voo para o alto / meio do monitor CRT
        time.sleep(3.5)
        page.screenshot(path='C:/Users/Lav/.gemini/antigravity/brain/fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce/spark_flight_2.png')
        print('Flight 2 captured')
        
        # 3. Pairando no ar ou soltando piada geek
        time.sleep(4.5)
        page.screenshot(path='C:/Users/Lav/.gemini/antigravity/brain/fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce/spark_flight_3.png')
        print('Flight 3 captured')
        
        # 4. Mais um momento de patrulha aérea
        time.sleep(4.5)
        page.screenshot(path='C:/Users/Lav/.gemini/antigravity/brain/fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce/spark_flight_4.png')
        print('Flight 4 captured')

        browser.close()
    print('Flight and joke test completed!')

if __name__ == '__main__':
    run()
