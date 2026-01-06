# Best practices

- Read the @README.md file to understand the project.
- Read the @AGENTS.md file to understand the best practices for this project.
- Every time you end a asked task, call the tool funcion "play_notification" to notify the user. Then make a commit. Only push when asked.

## Testing instructions

- Find the CI plan in the .github/workflows folder.
- Run `make precommit` to run every check defined for that package.
- From the package root you can just call `make precommit`. The commit should pass all tests before you merge.
- Fix any test or type errors until the whole suite is green.
- After moving files or changing imports, run `make precommit` to be sure rules still pass.
- Add or update tests for the code you change, even if nobody asked.

## Before PR

- Verify carefully if @README.md is updated or need to be updated.
- Based on last session, verify if @AGENTS.md ADR memories need to be updated or added new memories.

## PR instructions

- Title format: [<project_name>] <Title>
- Always run `make precommit` before committing.
