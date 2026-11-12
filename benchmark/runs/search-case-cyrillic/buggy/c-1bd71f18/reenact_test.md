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
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15304/.
Stay on localhost:15304 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Click the memo editor input on the home page to focus it
2. Type the memo content: "**Рецепты**", then a blank line, "**Сырники**:", then two blank lines, "**Блины**", a new line, "**Выход** 12-13 шт (большая сковородка), 12", a new line, "- молоко 400 (500) мл"
3. Click the Save button to create the memo
4. Click the search box
5. Type "блины" (lowercase б) into the search box
6. Press Enter to run the search
The reporter says the correct behavior is that the memo containing "Блины" is shown in the search results after searching for "блины", the same as when searching for "Блины". Verify that the memo containing "Блины" is shown in the search results after searching for "блины", the same as when searching for "Блины".
