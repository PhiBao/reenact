---
mode: testing
url: http://localhost:8091/
headless: true
max_steps: 25
tags: [reenact-ui, smoke]
---

# Reenact home

## The home page explains the product and lists case files
Open the page.
Verify the main heading reads "Every bug report, reenacted."
Verify a section titled "Case files" lists at least one case with a verdict label.
Click the first case in the list.
Verify the page shows a section called "The claim" and a section called "The reenactment".
