name: Pull Request
description: Submit a pull request
body:
  - type: markdown
    attributes:
      value: |
        Thank you for contributing to HTTP File Runner! Please fill out this template to help us review your changes.

  - type: dropdown
    id: change-type
    attributes:
      label: Type of Change
      description: What type of change does this PR introduce?
      options:
        - Bug fix (non-breaking change that fixes an issue)
        - New feature (non-breaking change that adds functionality)
        - Breaking change (fix or feature that would cause existing functionality to not work as expected)
        - Documentation update
        - Code refactoring
        - Performance improvement
        - Other
    validations:
      required: true

  - type: textarea
    id: description
    attributes:
      label: Description
      description: Please describe your changes in detail
      placeholder: What does this PR do? Why is this change needed?
    validations:
      required: true

  - type: textarea
    id: testing
    attributes:
      label: Testing
      description: How have you tested these changes?
      placeholder: |
        - [ ] Unit tests pass
        - [ ] Integration tests pass
        - [ ] Manual testing performed
        - [ ] Tested on multiple platforms
    validations:
      required: true

  - type: textarea
    id: related-issues
    attributes:
      label: Related Issues
      description: Link any related issues
      placeholder: |
        Fixes #123
        Closes #456
        Related to #789

  - type: checkboxes
    id: checklist
    attributes:
      label: Checklist
      description: Please confirm the following
      options:
        - label: My code follows the project's coding standards
          required: true
        - label: I have performed a self-review of my code
          required: true
        - label: I have commented my code where necessary
          required: true
        - label: I have made corresponding changes to the documentation
          required: false
        - label: My changes generate no new warnings
          required: true
        - label: I have added tests that prove my fix is effective or that my feature works
          required: false
        - label: New and existing unit tests pass locally with my changes
          required: true
