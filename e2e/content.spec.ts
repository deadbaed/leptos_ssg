import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
    await page.goto("/example-site/demo");
});

test("has title", async ({ page }) => {
    await expect(page).toHaveTitle(/^Demo/);
});

test("has title ending with site name", async ({ page }) => {
    await expect(page).toHaveTitle(/ - leptos_ssg$/);
});

test("has header", async ({ page }) => {
    expect(page.getByRole("heading", { name: "Demo of leptos_ssg" }));

    // TODO: posted date
});

test("has navigation links", async ({ page }) => {
    await expect(page.getByRole("link", { name: "Home" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Previous" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Next" })).toBeVisible();
});
