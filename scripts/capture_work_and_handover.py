import asyncio
from playwright.async_api import async_playwright

async def main():
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page(viewport={'width': 1440, 'height': 900})
        await page.goto('http://localhost:5180/', wait_until='networkidle')
        await asyncio.sleep(2)

        inp = await page.query_selector('input')
        if not inp:
            print('Input not found!')
            return

        # 1. Dispatch mission to Rusty
        print('1. Typing mission to Rusty...')
        await inp.click()
        await inp.type('@Rusty refatore o driver SQLite para WASM OPFS', delay=10)
        await asyncio.sleep(0.3)
        await page.keyboard.press('Enter')
        print('Dispatched to Rusty! Waiting for him to walk to Dev Lab chair and type...')

        # Wait 3.5s for Rusty to walk to desk (-3, 7) and sit on the gaming chair
        await asyncio.sleep(3.5)
        out1 = r'C:\Users\Lav\.gemini\antigravity\brain\fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce\rusty_seated_at_desk_typing.png'
        await page.screenshot(path=out1)
        print(f'Captured {out1}!')

        # 2. Test Pipeline Handover: Rusty delegates to Nova
        print('2. Dispatching handover to Nova...')
        await inp.click()
        await inp.type('@Rusty encaminhe para Nova refatore o analisador de AST', delay=10)
        await asyncio.sleep(0.3)
        await page.keyboard.press('Enter')
        print('Handover dispatched! Waiting for handover and Nova to sit and execute...')

        await asyncio.sleep(4.0)
        out2 = r'C:\Users\Lav\.gemini\antigravity\brain\fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce\pipeline_handover_success.png'
        await page.screenshot(path=out2)
        print(f'Captured {out2}!')

        await browser.close()

if __name__ == '__main__':
    asyncio.run(main())
