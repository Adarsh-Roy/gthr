# Homebrew Tap Setup

This directory contains files needed to set up a Homebrew tap repository.

## Steps to create the tap repository:

1. Create a new GitHub repository named `homebrew-gthr`
2. Copy the contents of this directory to that repository
3. The repository should have the following structure:

```
homebrew-gthr/
├── README.md
└── Formula/
    └── gthr.rb
```

## After creating the tap repository:

Users will be able to install gthr with:

```bash
brew tap adarsh-roy/gthr
brew install gthr
```

Or in one command:
```bash
brew install adarsh-roy/gthr/gthr
```