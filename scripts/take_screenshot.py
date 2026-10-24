import asyncio
from playwright.async_api import async_playwright

async def main():
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page(viewport={"width": 1440, "height": 900})
        print("Navigating to http://127.0.0.1:5173/...")
        await page.goto("http://127.0.0.1:5173/", wait_until="networkidle")
        # Wait a few seconds for Phaser canvas to load and agents to move
        await asyncio.sleep(4)
        output_path = r"C:\Users\Lav\.gemini\antigravity\brain\14580eb8-accb-470e-a9cf-2879d405ccdd\the_grid_metropolis.png"
        await page.screenshot(path=output_path)
        print(f"Screenshot saved to {output_path}")
        await browser.close()

if __name__ == "__main__":
    asyncio.run(main())
