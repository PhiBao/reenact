---
mode: testing
url: http://localhost:15304/auth
headless: true
max_steps: 40
tags: [reenact, issue-5559]
---

# Search is not case-idependent in non-EN locale

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the user is signed in and the main memo page is visible.

## Reproduce the report
Open http://localhost:15304/.
Stay on localhost:15304 for the whole run and do not open any other website.
Do these in order:
1. Create a new memo with the content: "**Рецепты** **Сырники**: **Блины** **Выход** 12-13 шт (большая сковородка), 12 - молоко 400 (500) мл"
2. Save the memo
3. Click the search box
4. Type "блины" into the search box
5. Press Enter to run the search
The reporter says the correct behavior is that the memo containing "**Блины**" appears in the search results when searching for "блины". Verify that the memo containing "**Блины**" appears in the search results when searching for "блины".
