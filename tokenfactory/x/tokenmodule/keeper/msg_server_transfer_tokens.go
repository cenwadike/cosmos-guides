package keeper

import (
	"context"

	"tokenfactory/x/tokenmodule/types"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/math"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

func (k msgServer) TransferTokens(goCtx context.Context, msg *types.MsgTransferTokens) (*types.MsgTransferTokensResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	// Check if the value exists
	valFound, isFound := k.GetDenom(
		ctx,
		msg.Denom,
	)
	if !isFound {
		return nil, errorsmod.Wrap(sdkerrors.ErrKeyNotFound, "denom does not exist")
	}

	// Checks if the the msg owner is the same as the current owner
	if msg.Owner != valFound.Owner {
		return nil, errorsmod.Wrap(sdkerrors.ErrUnauthorized, "incorrect owner")
	}

	// Ensure reciever address is valid
	_, err := sdk.AccAddressFromBech32(msg.To)
	if err != nil {
		return nil, errorsmod.Wrapf(sdkerrors.ErrInvalidAddress, "invalid From address (%s)", err)
	}

	// Parse amount
	var coins sdk.Coins
	coins = coins.Add(sdk.NewCoin(msg.Denom, math.NewInt(int64(msg.Amount))))

	// Transfer coins
	err = k.bankKeeper.SendCoins(ctx, sdk.AccAddress(msg.From), sdk.AccAddress(msg.To), coins)
	if err != nil {
		return nil, err
	}

	return &types.MsgTransferTokensResponse{}, nil
}
