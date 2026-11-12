---
mode: testing
url: http://localhost:15302/auth
headless: true
max_steps: 40
tags: [reenact, issue-5575]
---

# The nested checklist cannot display levels correctly

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15302/.
Stay on localhost:15302 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Click the memo editor input on the home page
2. Type the line '- [ ] task1' and press Enter
3. Type the line ' - [ ] task1.1' (indented by 4 spaces) and press Enter
4. Type the line ' - [ ] task 1.1.1' (indented by 8 spaces) and press Enter
5. Type the line ' - [ ] task1.2 ' (indented by 4 spaces) and press Enter
6. Type the line '- [ ] task2'
7. Click the Save button to save the memo
8. View the rendered memo in the memo list
The reporter says the correct behavior is that The rendered memo shows task1.1 and task1.2 indented one level under task1, and task 1.1.1 indented two levels under task1, with task2 at the top level, rather than all items appearing on the same level. Verify that The rendered memo shows task1.1 and task1.2 indented one level under task1, and task 1.1.1 indented two levels under task1, with task2 at the top level, rather than all items appearing on the same level.
