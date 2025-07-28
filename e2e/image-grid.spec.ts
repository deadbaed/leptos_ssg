import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
    await page.goto("/example-site/image-grid");
});

test("has image grid", async ({ page }) => {
    await expect(page.getByTestId('ImageGrid')).toBeVisible();
});
