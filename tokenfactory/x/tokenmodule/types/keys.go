package types

const (
	// ModuleName defines the module name
	ModuleName = "tokenmodule"

	// StoreKey defines the primary module store key
	StoreKey = ModuleName

	// MemStoreKey defines the in-memory store key
	MemStoreKey = "mem_tokenmodule"
)

var (
	ParamsKey = []byte("p_tokenmodule")
)

func KeyPrefix(p string) []byte {
	return []byte(p)
}
