import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
    await page.goto("/example-site/");
});

test("has title", async ({ page }) => {
    await expect(page).toHaveTitle(/leptos_ssg/);
});

test("has header", async ({ page }) => {
    await expect(page.getByRole("img", { name: "leptos_ssg logo" }))
        .toBeVisible();
    await expect(page.getByRole("heading")).toContainText("leptos_ssg");
    await expect(page.getByRole("main")).toContainText(
        "simple site to showcase leptos_ssg",
    );
});

test("has navigation links", async ({ page }) => {
    await expect(page.getByRole("link", { name: "RSS" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Website" })).toBeVisible();
});

test.fixme("has RSS feed", async ({ page }) => {
    await page.getByRole("link", { name: "RSS" }).click();

    // Expect blog UUID to be present
    await expect(page.getByText("00000000-0000-4000-0000-000000000000"))
        .toBeVisible();
});

test("contains build timestamp", async ({ page }) => {
    await expect(page.getByText("Page generated on")).toBeVisible();
});
