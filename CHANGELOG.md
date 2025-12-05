# Changelog

## [0.2.2] - Unreleased

### Added

- Exec environment
  - `BATL_PWD` - The path where battalion was executed

### Changed

- Configuration format
	- `restrict.[restrictor]`

## [0.2.1] - 2024-05-18

### Changed

- Commands
	- `workspace ls [filter]`
	- `repository archive <name>`
	- `repository publish <name>`
	- `repository fetch <name>`
	- `repository which <name>`
	- `repository exec [-n <name>] <script>`
	- `auth`

## [0.2.0] - 2024-05-08

### Changed

- Repositories as workspaces
- Clonable repositories
- Repository environment variables
- Workspace equivalency to repositories in some cases
- Configuration format
  - `repository.build` -> `scripts`
- Commands
	- `workspace cd`
	- `workspace which`
	- `workspace init --ref`
	- `repository clone`
	- `repository scaffold`
	- `repository env`
	- `link stats --get`
	- `add`
	- `remove/rm`

## [0.1.0] - 2023-05-02

### Added

- Commands
  - `link ls`
  - `link stats`
  - `link init`
  - `link delete`
  - `link run`
  - `repository ls`
  - `repository init`
  - `repository delete`
  - `workspace ls`
  - `workspace init`
  - `workspace delete`
  - `setup`
- Terminal styling
- First official config format
