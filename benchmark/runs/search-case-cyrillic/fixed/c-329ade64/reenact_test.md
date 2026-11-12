---
mode: testing
url: http://localhost:15305/auth
headless: true
max_steps: 40
tags: [reenact, issue-5559]
---

# Search is not case-idependent in non-EN locale

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15305/.
Stay on localhost:15305 for the whole run and do not open any other website.
Do these in order:
1. Click the memo editor input on the home page
2. Type the memo content: **Рецепты** **Сырники**: **Блины** **Выход** 12-13 шт (большая сковородка), 12 - молоко 400 (500) мл
3. Click the Save button to create the memo
4. Click the search input
5. Type "блины" into the search input
6. Press Enter to run the search
The reporter says the correct behavior is that the memo containing "Блины" is shown in the search results when searching for lowercase "блины". Verify that the memo containing "Блины" is shown in the search results when searching for lowercase "блины".
