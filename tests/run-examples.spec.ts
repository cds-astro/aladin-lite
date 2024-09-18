import {test, expect} from '@playwright/test';

async function open(page, exampleName) {
    await page.goto(`http://localhost:5173/tests/${exampleName}.html`);
    await page.setViewportSize({height: 600, width: 800});
}

const tests = [
    // Check if ui elements appear correctly when the boolean option are on
    {
        name: "UI options on",
        path: "../examples/al-ui-on"
    },
    // Check only the image view on a local stored HiPS
    {
        name: "local HiPS",
        path: "../examples/al-ui-off"
    },
    // Check display of a FITS image
    {
        name: "local FITS image",
        path: "../examples/al-displayFITS"
    },
    // Plot a votable coming from VizieR and display labels on its sources
    {
        name: "named labels catalogue",
        path: "../examples/al-onames-labels"
    },
    // Multiple HiPS surveys referenced by an ID string (old v2 way)
    {
        name: "multiple HiPS display each referenced by an ID string",
        path: "../examples/al-cfht"
    },
    // GW as moc, test multiple rendering modes: perimeter, filled and hpx cells
    {
        name: "GW MOC plot",
        path: "../examples/al-gw"
    }
];

(async () => {
    for (let t of tests) {
        await test(t.name, async ({ page }) => {
            await open(page, t.path);
            await page.waitForLoadState("networkidle")
        
            expect(
                await page.locator('canvas').nth(1).screenshot({
                    type: "jpeg",
                    quality: 70,
                })
            ).toMatchSnapshot({
                maxDiffPixels: 30
            });
        });
    } 
})()
 

