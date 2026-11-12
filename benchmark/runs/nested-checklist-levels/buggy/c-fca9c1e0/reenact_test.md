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
Do these in order:
1. Click the memo editor input on the home page
2. Type the line '- [ ] task1' and press Enter
3. Type the line ' - [ ] task1.1' (indented by 4 spaces) and press Enter
4. Type the line ' - [ ] task 1.1.1' (indented by 8 spaces) and press Enter
5. Type the line ' - [ ] task1.2 ' (indented by 4 spaces) and press Enter
6. Type the line '- [ ] task2'
7. Click the Save button to create the memo
8. View the rendered memo in the memo list
The reporter says the correct behavior is that The rendered memo shows the checklist items nested by level: task1.1 and task1.2 are indented under task1, task 1.1.1 is indented further under task1.1, and task2 is at the top level. Verify that The rendered memo shows the checklist items nested by level: task1.1 and task1.2 are indented under task1, task 1.1.1 is indented further under task1.1, and task2 is at the top level.
