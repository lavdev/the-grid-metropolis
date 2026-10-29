import asyncio
from playwright.async_api import async_playwright

async def main():
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page(viewport={'width': 1440, 'height': 900})
        page.on('console', lambda msg: print('PAGE LOG:', msg.text))
        await page.goto('http://localhost:5180/', wait_until='networkidle')
        await asyncio.sleep(2)

        # 1. Base CRT Terminal with SPARK-01 Idle
        out1 = r'C:\Users\Lav\.gemini\antigravity\brain\fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce\crt_terminal_base.png'
        await page.screenshot(path=out1)
        print(f'Captured {out1} (Base CRT Terminal)!')

        # 2. Type and dispatch user command from sketch: "preciso analisar os logs da rede agora"
        inp = await page.query_selector('input[placeholder*="Digite seu comando"]')
        if inp:
            print('Typing user command into CLI...')
            await inp.click()
            await inp.type('preciso analisar os logs da rede agora', delay=10)
            await asyncio.sleep(0.3)
            await page.keyboard.press('Enter')
            print('Command dispatched! Waiting 1s to capture Thinking/Waveform state...')

        # 3. Capture streaming/speaking waveform state
        await asyncio.sleep(1.2)
        out2 = r'C:\Users\Lav\.gemini\antigravity\brain\fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce\crt_terminal_streaming.png'
        await page.screenshot(path=out2)
        print(f'Captured {out2} (Streaming / Speaking Waveform)!')

        # 4. Wait for completion and capture Celebrate Confetti state
        await asyncio.sleep(5.0)
        out3 = r'C:\Users\Lav\.gemini\antigravity\brain\fecb0741-c59c-4f5f-a76b-3a7a0c9a38ce\crt_terminal_celebrate.png'
        await page.screenshot(path=out3)
        print(f'Captured {out3} (Celebration / Success)!')

        await browser.close()

if __name__ == '__main__':
    asyncio.run(main())
