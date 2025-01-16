package keeper

import (
	"tokenfactory/x/tokenmodule/types"
)

var _ types.QueryServer = Keeper{}
