import asyncio
from playwright.async_api import async_playwright

async def main():
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page(viewport={'width': 1440, 'height': 900})
        page.on('console', lambda msg: print('PAGE LOG:', msg.text))
        await page.goto('http://localhost:5180/', wait_until='networkidle')
        await asyncio.sleep(2)

        inp = await page.query_selector('input')
        if not inp:
            print('Input element not found!')
            return

        # 1. Dispatch mission to Rusty
        print('--- 1. Dispatching mission to Rusty ---')
        await inp.click()
        await inp.type('@Rusty refatore o driver SQLite para WASM OPFS', delay=10)
        await asyncio.sleep(0.3)
        await page.keyboard.press('Enter')
        print('Dispatched to Rusty! Waiting 4.2s for him to walk and sit at the desk...')

        await asyncio.sleep(4.2)
        pos1 = await page.evaluate('''() => {
            const bridge = window.__gridGameBridge;
            const rusty = bridge.getScene().agentManager.getAgent('agent-codex-01');
            return {
                cartX: rusty.cartX,
                cartY: rusty.cartY,
                status: rusty.status,
                direction: rusty.direction
            };
        }''')
        print('Rusty position and state:', pos1)

        out1 = r'C:\Users\Lav\.gemini\antigravity\brain\fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce\rusty_sitting_at_desk_verified.png'
        await page.screenshot(path=out1)
        print(f'Captured {out1}!')

        # 2. Dispatch Pipeline Handover: Rusty -> Nova
        print('--- 2. Dispatching Handover to Nova ---')
        await inp.click()
        await inp.type('@Rusty encaminhe para Nova refatore o analisador de AST', delay=10)
        await asyncio.sleep(0.3)
        await page.keyboard.press('Enter')
        print('Handover dispatched! Waiting 5.5s for handover interaction...')

        await asyncio.sleep(5.5)
        pos2 = await page.evaluate('''() => {
            const bridge = window.__gridGameBridge;
            const manager = bridge.getScene().agentManager;
            const rusty = manager.getAgent('agent-codex-01');
            const nova = manager.getAgent('agent-nova-03');
            return {
                rusty: { x: rusty.cartX, y: rusty.cartY, status: rusty.status },
                nova: { x: nova.cartX, y: nova.cartY, status: nova.status }
            };
        }''')
        print('Agents after handover:', pos2)

        out2 = r'C:\Users\Lav\.gemini\antigravity\brain\fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce\pipeline_handover_verified.png'
        await page.screenshot(path=out2)
        print(f'Captured {out2}!')

        await browser.close()

if __name__ == '__main__':
    asyncio.run(main())
