import asyncio
from playwright.async_api import async_playwright

async def main():
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page(viewport={'width': 1440, 'height': 900})
        await page.goto('http://localhost:5180/', wait_until='networkidle')
        await asyncio.sleep(3)
        
        # 1. Main view with Global Chat Bar
        out1 = r'C:\Users\Lav\.gemini\antigravity\brain\fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce\habbo_chat_hud.png'
        await page.screenshot(path=out1)
        print('Captured 1:', out1)

        # 2. Click on Missoes button to open Backlog
        buttons = await page.query_selector_all('button')
        for b in buttons:
            txt = await b.inner_text()
            if 'Missões' in txt:
                await b.click()
                break
        await asyncio.sleep(1)
        out2 = r'C:\Users\Lav\.gemini\antigravity\brain\fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce\mission_backlog_linear.png'
        await page.screenshot(path=out2)
        print('Captured 2:', out2)
        
        # Close backlog
        for b in await page.query_selector_all('button'):
            txt = await b.inner_text()
            if 'Fechar Painel' in txt:
                await b.click()
                break
        await asyncio.sleep(0.5)

        # 3. Type a command in the chat bar and send
        input_el = await page.query_selector('input[placeholder*="Instrua a equipe"]')
        if input_el:
            await input_el.fill('@Rusty refatore o driver SQLite para WASM OPFS')
            await asyncio.sleep(0.5)
            for b in await page.query_selector_all('button'):
                txt = await b.inner_text()
                if 'Despachar' in txt:
                    await b.click()
                    break
            # Wait for agent reaction and CoT thinking
            await asyncio.sleep(3)

        # 4. Open Agent Drawer by clicking on an agent in canvas or dispatching click event
        await page.evaluate("""() => {
            window.dispatchEvent(new CustomEvent('open_agent_drawer', { detail: 'agent-codex-01' }));
        }""")
        
        # Or click directly on canvas near Rusty (approx center)
        await page.mouse.click(720, 480)
        await asyncio.sleep(1)
        
        out3 = r'C:\Users\Lav\.gemini\antigravity\brain\fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce\agent_continuous_chat.png'
        await page.screenshot(path=out3)
        print('Captured 3:', out3)

        await browser.close()

if __name__ == '__main__':
    asyncio.run(main())
