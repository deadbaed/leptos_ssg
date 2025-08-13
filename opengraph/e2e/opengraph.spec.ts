import { APIRequestContext, expect, test } from "@playwright/test";

async function getOpengraphImage(request: APIRequestContext, url: string) {
    const opengraphImage = `${url}/opengraph.png`;
    console.log(`Getting opengraph image ${opengraphImage}`);
    const response = await request.get(opengraphImage);
    expect(response.ok()).toBeTruthy();

    const contentType = response.headers()['content-type'];
    expect(contentType).toMatch(/^image\//);
}

test("homepage has opengraph image", async ({ request }) => {
    await getOpengraphImage(request, "/example-site");
});

test("all content pages have opengraph image", async ({ page, request }) => {
    await page.goto("/example-site");

    // Get all links to content
    const contentList = page.getByTestId("content-list");
    // Use .all() to get an array of Locator handles
    const listItems = await contentList.locator(page.getByRole("link")).all();

    // Extract href of all links, make sure they are non null
    const links = (await Promise.all(listItems.map(async (item): Promise<string | null> => {
        const href = await item.getAttribute("href");
        return href;
    }
    ))).filter(el => el != null);

    for (const link of links) {
        await getOpengraphImage(request, link);
    }

});
