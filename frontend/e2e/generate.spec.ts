import { test, expect } from "@playwright/test";

test.describe("Platerator", () => {
  test("generates and downloads a plate model", async ({ page }) => {
    await page.goto("/");

    // Verify we're on the right page
    await expect(page.locator('[data-slot="card-title"]')).toContainText(
      "Platerator"
    );

    // Fill in the form with valid values
    await page.fill('[name="boltSpacing"]', "60");
    await page.fill('[name="boltDiameter"]', "10");
    await page.fill('[name="bracketHeight"]', "400");
    await page.fill('[name="bracketWidth"]', "300");
    await page.fill('[name="pinDiameter"]', "10");
    await page.fill('[name="pinCount"]', "6");
    await page.fill('[name="plateThickness"]', "8");

    // Click generate and wait for loading state
    await page.click('button[type="submit"]');
    await expect(page.locator('button[type="submit"]')).toContainText(
      "Generating..."
    );

    // Wait for success message (can take a while due to zoo CLI)
    await expect(page.locator("text=Model generated successfully")).toBeVisible(
      { timeout: 60000 }
    );

    // Verify the model viewer loaded with session-specific URL
    const modelViewer = page.locator("model-viewer");
    await expect(modelViewer).toHaveAttribute("src", /\/api\/download\/gltf\//, {
      timeout: 10000,
    });

    // Verify download button has correct href
    const downloadLink = page.locator('a:has-text("Download STEP")');
    await expect(downloadLink).toBeVisible();
    await expect(downloadLink).toHaveAttribute(
      "href",
      /\/api\/download\/step\//
    );

    // Test the download works
    const [download] = await Promise.all([
      page.waitForEvent("download"),
      downloadLink.click(),
    ]);

    expect(download.suggestedFilename()).toBe("actuator_plate.step");
  });

  // TODO: Add error handling test once error display is improved
  // Currently, error classes are compiled by Tailwind v4 and hard to query
});
