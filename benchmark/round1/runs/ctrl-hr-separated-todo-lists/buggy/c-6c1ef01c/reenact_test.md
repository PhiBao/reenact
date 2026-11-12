---
mode: testing
url: http://localhost:15330/auth
headless: true
max_steps: 40
tags: [reenact, issue-5635]
---

# Checkbox state interference between todo lists separated by horizontal rules

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15330/.
Stay on localhost:15330 for the whole run and do not open any other website.
Do these in order:
1. Click on the memo editor input on the home page
2. Type the line '## test todo checkbox' and press Enter twice
3. Type '- [ ] todo 1-1' and press Enter
4. Type '- [ ] todo 1-2'
5. Press Enter twice to leave the list, then type '---'
6. Press Enter twice, then type '- [ ] todo 2-1'
7. Press Enter and type '- [ ] todo 2-2'
8. Click the Save button to save the memo
9. Click the checkbox next to 'todo 1-2' in the saved memo to mark it completed
10. Look at the checkboxes next to 'todo 2-1' and 'todo 2-2' in the second list
The reporter says the correct behavior is that After checking 'todo 1-2', only 'todo 1-2' appears checked; 'todo 1-1', 'todo 2-1' and 'todo 2-2' all remain unchecked. Verify that After checking 'todo 1-2', only 'todo 1-2' appears checked; 'todo 1-1', 'todo 2-1' and 'todo 2-2' all remain unchecked.
