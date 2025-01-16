package types

import (
	errorsmod "cosmossdk.io/errors"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

var _ sdk.Msg = &MsgTransferTokens{}

func NewMsgTransferTokens(owner string, denom string, from string, to string, amount int32) *MsgTransferTokens {
	return &MsgTransferTokens{
		Owner:  owner,
		Denom:  denom,
		From:   from,
		To:     to,
		Amount: amount,
	}
}

func (msg *MsgTransferTokens) ValidateBasic() error {
	_, err := sdk.AccAddressFromBech32(msg.Owner)
	if err != nil {
		return errorsmod.Wrapf(sdkerrors.ErrInvalidAddress, "invalid owner address (%s)", err)
	}
	return nil
}
