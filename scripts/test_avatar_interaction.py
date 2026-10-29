import asyncio
import os
from playwright.async_api import async_playwright

async def main():
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page(viewport={'width': 1440, 'height': 900})
        await page.goto('http://localhost:5180/', wait_until='networkidle')
        await asyncio.sleep(2)

        # 1. Capture base headquarters with updated avatar scale & proportions
        out_base = r'C:\Users\Lav\.gemini\antigravity\brain\fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce\avatar_scale_hq.png'
        await page.screenshot(path=out_base)
        print('Captured base screenshot with updated avatar scale.')

        # 2. Dispatch instruction to Rusty via typing + Enter
        input_el = await page.query_selector('input[placeholder*="Instrua a equipe"]')
        if input_el:
            await input_el.click()
            await input_el.fill('@Rusty refatore o driver SQLite para WASM OPFS')
            await asyncio.sleep(0.3)
            await page.keyboard.press('Enter')
            print('Dispatched task to Rusty via Enter key!')

        # 3. Wait for Rusty to walk to Dev Lab gaming chair and sit down to work
        await asyncio.sleep(4.0)
        out_work = r'C:\Users\Lav\.gemini\antigravity\brain\fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce\rusty_devlab_seated_working.png'
        await page.screenshot(path=out_work)
        print('Captured screenshot of Rusty seated at Dev Lab desk working!')

        # 4. Test Handover / Delegation: Rusty forwards task to Nova
        if input_el:
            await input_el.click()
            await input_el.fill('@Rusty encaminhe para Nova otimizar os buffers de memória')
            await asyncio.sleep(0.3)
            await page.keyboard.press('Enter')
            print('Dispatched handover instruction from Rusty to Nova!')

        # Wait for handover animation and Nova working
        await asyncio.sleep(5.0)
        out_handover = r'C:\Users\Lav\.gemini\antigravity\brain\fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce\pipeline_handover_devlab.png'
        await page.screenshot(path=out_handover)
        print('Captured screenshot of pipeline handover in Dev Lab!')

        await browser.close()

if __name__ == '__main__':
    asyncio.run(main())
