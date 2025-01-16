package tokenmodule_test

import (
	"testing"

	keepertest "tokenfactory/testutil/keeper"
	"tokenfactory/testutil/nullify"
	tokenmodule "tokenfactory/x/tokenmodule/module"
	"tokenfactory/x/tokenmodule/types"

	"github.com/stretchr/testify/require"
)

func TestGenesis(t *testing.T) {
	genesisState := types.GenesisState{
		Params: types.DefaultParams(),

		DenomList: []types.Denom{
			{
				Denom: "0",
			},
			{
				Denom: "1",
			},
		},
		// this line is used by starport scaffolding # genesis/test/state
	}

	k, ctx := keepertest.TokenmoduleKeeper(t)
	tokenmodule.InitGenesis(ctx, k, genesisState)
	got := tokenmodule.ExportGenesis(ctx, k)
	require.NotNil(t, got)

	nullify.Fill(&genesisState)
	nullify.Fill(got)

	require.ElementsMatch(t, genesisState.DenomList, got.DenomList)
	// this line is used by starport scaffolding # genesis/test/assert
}
