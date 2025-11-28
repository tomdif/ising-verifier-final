```
package types

import "errors"

var (
	ErrInvalidAddress  = errors.New("invalid address")
	ErrInvalidRequest  = errors.New("invalid request")
	ErrInvalidParams   = errors.New("invalid params")
	ErrJobNotFound     = errors.New("job not found")
	ErrLigandNotFound  = errors.New("ligand not found")
	ErrAlreadyVerified = errors.New("ligand already verified")
	ErrDuplicate       = errors.New("duplicate submission")
	ErrDisputed        = errors.New("ligand is disputed")
	ErrNoWorkAvailable = errors.New("no work available")
	ErrUnauthorized    = errors.New("unauthorized")
)
