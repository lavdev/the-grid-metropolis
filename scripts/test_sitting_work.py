import asyncio
from playwright.async_api import async_playwright

async def main():
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page(viewport={'width': 1440, 'height': 900})
        await page.goto('http://localhost:5180/', wait_until='networkidle')
        await asyncio.sleep(2)
        
        # 1. Type instruction in chat bar
        input_el = await page.query_selector('input[placeholder*="Instrua a equipe"]')
        if input_el:
            await input_el.fill('@Rusty refatore o driver SQLite para WASM OPFS')
            await asyncio.sleep(0.5)
            buttons = await page.query_selector_all('button')
            for b in buttons:
                txt = await b.inner_text()
                if 'Despachar' in txt:
                    await b.click()
                    print('Dispatched task to Rusty!')
                    break
                
        # 2. Wait for Rusty to walk to Dev Lab desk and sit on chair
        await asyncio.sleep(4.5)
        out1 = r'C:\Users\Lav\.gemini\antigravity\brain\fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce\rusty_sitting_working.png'
        await page.screenshot(path=out1)
        print('Captured screenshot of Rusty working at desk!')

        await browser.close()

if __name__ == '__main__':
    asyncio.run(main())
