# Token Factory Tutorial

Welcome to the Token Factory tutorial! In this guide, you'll learn how to create and manage custom tokens on the Cosmos blockchain using the Ignite CLI. 

## Objective
By the end of this tutorial, you'll have a fully functional Token Factory module that allows you to create, mint, and transfer tokens within your blockchain.

## Prerequisites
Before diving in, ensure you have the following:
1. **A computer** with a compatible operating system (Linux, macOS, or Windows).
2. **Basic knowledge** of blockchain concepts and the Go programming language.
3. **An active internet connection** to download and install necessary tools.

## Tools and Technologies
- **Go**: The programming language for developing Cosmos SDK modules.
- **Ignite CLI**: A powerful tool for scaffolding and managing Cosmos SDK-based blockchains.
- **Cosmos SDK**: A framework for building custom blockchains on the Cosmos network.

## Steps Overview
1. **Setting up Your Development Environment**: Install Go and Ignite CLI.
2. **Scaffolding Your Blockchain**: Use Ignite CLI to create a new blockchain project.
3. **Implementing the Token Factory Module**: Define data structures, messages, and keeper logic for the Token Factory.
4. **Expand Token Factory Functionalities**: Add mint and transfer logic. 
5. **Building and Serving Your Blockchain**: Build the blockchain and run a local server to test the Token Factory module.
6. **Interact with Token Factory Blockchain**: Use CLI commands to create, mint, and transfer tokens.

Ready to build your very own Token Factory? Let's get started and dive into the world of custom tokens on the Cosmos blockchain!

## 1. Setting up Your Development Environment

### Install Go

- **Download Go**
  - Visit the [Go download page](https://go.dev/doc/install) and select the appropriate version for your operating system (Linux, macOS, or Windows).
  - Follow the installation instructions provided on the page.

- **Verify Installation**
  - Open your terminal or command prompt and type:
    ```sh
    go version
    ```
  - You should see the installed version of Go.

### Install Ignite CLI

- **Download and Install**
  - Open your terminal and run the following command to download and install Ignite CLI:
    ```sh
    curl https://get.ignite.com/cli | bash
    ```
  - This will install the Ignite CLI binary in `/usr/local/bin`.

- **Verify Installation**
  - To verify the installation, run:
    ```sh
    ignite version
    ```
  - You should see the version of Ignite CLI that was installed.

## 2. Scaffolding Your Blockchain

To scaffold your blockchain using Ignite CLI, follow these steps:

- **Open Your Terminal**
  Make sure you have your terminal open and navigate to the directory where you want to create your project.

- **Scaffold a New Blockchain**
  - Run the following command to create a new blockchain project:
    ```sh
    ignite scaffold chain tokenfactory --no-module
    ```

- **Navigate to Your Project Directory**
  - Change directory to your newly created project:
    ```sh
    cd tokenfactory
    ```

## 3. Implementing the Token Factory Module

Next, we'll scaffold a new module for your token factory. This module will depend on the Cosmos SDK's `bank` and `auth` modules, which provide essential functionalities like account access and token management.

- **Scaffold the Token Factory Module**
  - Run the following command to create a token module:
    ```sh
    ignite scaffold module tokenModule --dep account,bank
    ```
  The successful execution of this command will be confirmed with a message indicating that the tokenModule has been created.

- **Defining Denom Data Structure**

To manage denoms within your token factory, define their structure using an Ignite map. This will store the data as key-value pairs.

  - Run the following command to create the denom data structure:
    ```sh
    ignite scaffold map Denom description:string ticker:string precision:int url:string maxSupply:int supply:int canChangeMaxSupply:bool --signer owner --index denom --module tokenModule
    ```

  Review the `proto/tokenfactory/tokenModule/denom.proto` file to see the scaffolding results, which include modifications to various files indicating successful creation of the denom structure.

- **Proto Definition Updates**

  Start by updating the structure of a new token denom in `proto/tokenfactory/tokenModule/tx.proto`.

  - For `MsgCreateDenom`:

    - Remove int32 supply = 8; and adjust the field order so canChangeMaxSupply becomes the 8th field.

    - Resulting `MsgCreateDenom` message:
      ```go
        message MsgCreateDenom {
          option (cosmos.msg.v1.signer) = "owner";
          string owner              = 1;
          string denom              = 2;
          string description        = 3;
          string ticker             = 4;
          int32  precision          = 5;
          string url                = 6;
          int32  maxSupply          = 7;
          bool   canChangeMaxSupply = 8; // <---- supply was removed
        }
      ```
    
  - For `MsgUpdateDenom`:

    - Omit `string ticker = 4;`, `int32 precision = 5;`, and `int32 supply = 8;`, and reorder the remaining fields.

    - Resulting `MsgUpdateDenom` message:
      ```go
      message MsgUpdateDenom {
        option (cosmos.msg.v1.signer) = "owner";
        string owner              = 1;
        string denom              = 2;
        string description        = 3;
        string url                = 4;
        int32  maxSupply          = 5;
        bool   canChangeMaxSupply = 6;
      }
      ```

- **Types Updates**

When creating new denoms, they initially have no supply. The supply is determined only when tokens are minted.

  -  Navigate to `x/tokenmodule/types/messages_denom.go` and do the following: 

      - Remove the `supply` parameter from `NewMsgCreateDenom`.

      - Update `NewMsgUpdateDenom` to exclude unchangeable parameters like `ticker`, `precision`, and `supply`.

      - Implement basic input validation in `x/tokenmodule/types/messages_denom.go` like so:
        ```go
          func (msg *MsgCreateDenom) ValidateBasic() error {
            _, err := sdk.AccAddressFromBech32(msg.Owner)
            if err != nil {
              return errorsmod.Wrapf(sdkerrors.ErrInvalidAddress, "invalid owner address (%s)", err)
            }

            // Ensure tiker is between 3-10 characters.
            tickerLength := len(msg.Ticker)
            if tickerLength < 3 {
              return errorsmod.Wrapf(sdkerrors.ErrInvalidRequest, "Ticker length must be at least 3 chars long")
            }
            if tickerLength > 10 {
              return errorsmod.Wrapf(sdkerrors.ErrInvalidRequest, "Ticker length must be less than 11 chars long maximum")
            }

            // Ensure max supply is greater than zero.
            if msg.MaxSupply == 0 {
              return errorsmod.Wrapf(sdkerrors.ErrInvalidRequest, "Max Supply must be greater than 0")
            }

            return nil
          }
        ```

        ```go
        func (msg *MsgUpdateDenom) ValidateBasic() error {
          _, err := sdk.AccAddressFromBech32(msg.Owner)
          if err != nil {
              return sdkerrors.Wrapf(sdkerrors.ErrInvalidAddress, "invalid owner address (%s)", err)
          }
          if msg.MaxSupply == 0 {
              return sdkerrors.Wrapf(sdkerrors.ErrInvalidRequest, "Max Supply must be greater than 0")
          }
          return nil
        }
        ```

- **Keeper Logic**

  The keeper is where you define the business logic for manipulating the database and writing to the key-value store.

  - Navigate to `x/tokenmodule/keeper/msg_server_denom.go` and do the following:
  
    - Update `CreateDenom()` to include logic for creating unique denoms.
      - Modify the error message to point to existing denoms.
      - Set `Supply` to 0.

    - Modify `UpdateDenom()` to verify ownership and manage `max supply`:

  - `x/tokenmodule/keeper/msg_server_denom.go` should look like so:
      ```go
          func (k msgServer) CreateDenom(goCtx context.Context, msg *types.MsgCreateDenom) (*types.MsgCreateDenomResponse, error) {
            ctx := sdk.UnwrapSDKContext(goCtx)

            // Check if the denom already exists
            _, isFound := k.GetDenom(
              ctx,
              msg.Denom,
            )
            if isFound {
              return nil, errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "Denom already exist")
            }

            // update state
            var zero int32 = 0
            var denom = types.Denom{
              Owner:              msg.Owner,
              Denom:              msg.Denom,
              Description:        msg.Description,
              Ticker:             msg.Ticker,
              Precision:          msg.Precision,
              Url:                msg.Url,
              MaxSupply:          msg.MaxSupply,
              Supply:             zero,
              CanChangeMaxSupply: msg.CanChangeMaxSupply,
            }

            k.SetDenom(
              ctx,
              denom,
            )
            return &types.MsgCreateDenomResponse{}, nil
          }

          func (k msgServer) UpdateDenom(goCtx context.Context, msg *types.MsgUpdateDenom) (*types.MsgUpdateDenomResponse, error) {
            ctx := sdk.UnwrapSDKContext(goCtx)

            // Check if the value exists
            valFound, isFound := k.GetDenom(
              ctx,
              msg.Denom,
            )
            if !isFound {
              return nil, errorsmod.Wrap(sdkerrors.ErrKeyNotFound, "Denom to update not found")
            }

            // Checks if the msg owner is the same as the current owner
            if msg.Owner != valFound.Owner {
              return nil, errorsmod.Wrap(sdkerrors.ErrUnauthorized, "Incorrect owner")
            }

            if !valFound.CanChangeMaxSupply {
              return nil, errorsmod.Wrap(sdkerrors.ErrUnauthorized, "Cannot change maxsupply")
            }

            if valFound.MaxSupply >= msg.MaxSupply {
              return nil, errorsmod.Wrap(sdkerrors.ErrUnauthorized, "Max supply already reached")
            }

            if !valFound.CanChangeMaxSupply && msg.CanChangeMaxSupply {
              return nil, errorsmod.Wrap(sdkerrors.ErrUnauthorized, "Cannot revert change maxsupply flag")
            }

            // Update state
            var denom = types.Denom{
              Owner:              msg.Owner,
              Denom:              msg.Denom,
              Description:        msg.Description,
              Ticker:             valFound.Ticker,
              Precision:          valFound.Precision,
              Url:                msg.Url,
              MaxSupply:          msg.MaxSupply,
              Supply:             valFound.Supply,
              CanChangeMaxSupply: msg.CanChangeMaxSupply,
            }

            k.SetDenom(ctx, denom)

            return &types.MsgUpdateDenomResponse{}, nil
          }
      ```

- **Bank and Auth Expected Keeper Interface**
  Since your module relies on the `auth` and `bank` modules, specify which of their functions your module can access.

  - Navigate to `x/tokenmodule/types/expected_keepers.go` and update the file like so:

    ```go
      package types

      import (
        "context"

        sdk "github.com/cosmos/cosmos-sdk/types"
      )

      // AccountKeeper defines the expected interface for the Account module.
      type AccountKeeper interface {
        GetModuleAddress(moduleName string) sdk.AccAddress
        GetModuleAccount(ctx context.Context, moduleName string) sdk.ModuleAccountI
        GetAccount(ctx context.Context, addr sdk.AccAddress) sdk.AccountI
      }

      type BankKeeper interface {
        SendCoins(ctx context.Context, fromAddr sdk.AccAddress, toAddr sdk.AccAddress, amt sdk.Coins) error
        MintCoins(ctx context.Context, moduleName string, amt sdk.Coins) error
        SpendableCoins(ctx context.Context, addr sdk.AccAddress) sdk.Coins
      }

      // ParamSubspace defines the expected Subspace interface for parameters.
      type ParamSubspace interface {
        Get(context.Context, []byte, interface{})
        Set(context.Context, []byte, interface{})
      }
    ```

## 4. Expand Token Factory Functionalities

Now we can focus on enhancing the token factory module by adding two critical messages: `MintTokens` and `TransferTokens`. These functionalities are key to managing tokens within your blockchain.

- **Scaffolding New Messages**

  - **MintTokens:**:
    This message allows the minting of new tokens and their allocation to a specified recipient. 
    The necessary inputs are the `denom`, the `amount` to mint, and the `recipient`'s address.

    Scaffold this message with:

    ```sh
      ignite scaffold message MintTokens denom:string amount:int recipient:string --module tokenModule --signer owner
    ```

  - **TransferTokens**:
    This message facilitates the transfer of a denom from sender's address to recipient's address. 
    It requires the denom name, sender's address, recipient's address, and amount.

    Scaffold this message with:

    ```sh
      ignite scaffold message TransferTokens denom:string from:string to:string amount:int --module tokenModule --signer owner
    ```

- **Implementing Logic for New Messages**

  - **In the MintTokens Functionality**:

    Locate `x/tokenmodule/keeper/msg_server_mint_tokens.go` and do the following:

    - Verifying the existence and ownership of the denom.
    - Ensuring minting does not exceed the maximum supply.
    - Minting the specified amount and sending it to the recipient.

    `x/tokenmodule/keeper/msg_server_mint_tokens.go` should look like so:

      ```go
        package keeper

        import (
          "context"

          "tokenfactory/x/tokenmodule/types"

          errorsmod "cosmossdk.io/errors"
          "cosmossdk.io/math"
          sdk "github.com/cosmos/cosmos-sdk/types"
          sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
        )

        func (k msgServer) MintTokens(goCtx context.Context, msg *types.MsgMintTokens) (*types.MsgMintTokensResponse, error) {
          ctx := sdk.UnwrapSDKContext(goCtx)

          // Check if the value exists
          valFound, isFound := k.GetDenom(
            ctx,
            msg.Denom,
          )
          if !isFound {
            return nil, errorsmod.Wrap(sdkerrors.ErrKeyNotFound, "Denom not found")
          }

          // Checks if the msg owner is the same as the current owner
          if msg.Owner != valFound.Owner {
            return nil, errorsmod.Wrap(sdkerrors.ErrUnauthorized, "Incorrect owner")
          }

          // Ensure max supply is not exceeded
          var newSupply int32 = valFound.Supply + msg.Amount
          if newSupply > valFound.MaxSupply {
            return nil, errorsmod.Wrap(sdkerrors.ErrTxDecode, "Can not mint more than total supply into circulation")
          }

          // Get module address
          moduleAcct := k.accountKeeper.GetModuleAddress(types.ModuleName)

          // Validate recipient address
          recipientAddress, err := sdk.AccAddressFromBech32(msg.Recipient)
          if err != nil {
            return nil, err
          }

          // Mint new denoms
          var mintCoins sdk.Coins
          mintCoins = mintCoins.Add(sdk.NewCoin(msg.Denom, math.NewInt(int64(msg.Amount))))

          if err := k.bankKeeper.MintCoins(ctx, types.ModuleName, mintCoins); err != nil {
            return nil, err
          }
          if err := k.bankKeeper.SendCoins(ctx, moduleAcct, recipientAddress, mintCoins); err != nil {
            return nil, err
          }

          // Update state
          var denom = types.Denom{
            Owner:              valFound.Owner,
            Denom:              valFound.Denom,
            Description:        valFound.Description,
            MaxSupply:          valFound.MaxSupply,
            Supply:             valFound.Supply + msg.Amount,
            Precision:          valFound.Precision,
            Ticker:             valFound.Ticker,
            Url:                valFound.Url,
            CanChangeMaxSupply: valFound.CanChangeMaxSupply,
          }

          k.SetDenom(
            ctx,
            denom,
          )

          return &types.MsgMintTokensResponse{}, nil
        }
      ```
  - **In the TransferTokens Functionality**:

    Locate `x/tokenmodule/keeper/msg_server_transfer_tokens.go` and do the following:

    - Checking if the denom exists.
    - Ensuring that the request comes from the current owner.
    - Transfer denom from sender's address to recipient's address.

    `x/tokenmodule/keeper/msg_server_transfer_tokens.go` should look like so:

      ```go
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

        // Convert From address
        fromAddr, err := sdk.AccAddressFromBech32(msg.From)
        if err != nil {
          return nil, errorsmod.Wrapf(sdkerrors.ErrInvalidAddress, "invalid From address: %v", err)
        }

        // Convert To address
        toAddr, err := sdk.AccAddressFromBech32(msg.To)
        if err != nil {
          return nil, errorsmod.Wrapf(sdkerrors.ErrInvalidAddress, "invalid To address: %v", err)
        }
        // Parse amount
        var coins sdk.Coins
        coins = coins.Add(sdk.NewCoin(msg.Denom, math.NewInt(int64(msg.Amount))))

        // Transfer coins
        err = k.bankKeeper.SendCoins(ctx, fromAddr, toAddr, coins)
        if err != nil {
          return nil, err
        }

        return &types.MsgTransferTokensResponse{}, nil
      }
      ```

- **Update CLI**

Navigate to `x/tokenmodule/module/autocli.go` and do the following:
  
  - Remove `supply` from `CreateDenom` RPC method.
  - Remove `ticker`, `precision`, and `supply` from `UpdateDenom` RPC method.

`x/tokenmodule/module/autocli.go` should look like so:

  ```go
    package tokenmodule

    import (
      autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"

      modulev1 "tokenfactory/api/tokenfactory/tokenmodule"
    )

    // AutoCLIOptions implements the autocli.HasAutoCLIConfig interface.
    func (am AppModule) AutoCLIOptions() *autocliv1.ModuleOptions {
      return &autocliv1.ModuleOptions{
        Query: &autocliv1.ServiceCommandDescriptor{
          Service: modulev1.Query_ServiceDesc.ServiceName,
          RpcCommandOptions: []*autocliv1.RpcCommandOptions{
            {
              RpcMethod: "Params",
              Use:       "params",
              Short:     "Shows the parameters of the module",
            },
            {
              RpcMethod: "DenomAll",
              Use:       "list-denom",
              Short:     "List all Denom",
            },
            {
              RpcMethod:      "Denom",
              Use:            "show-denom [id]",
              Short:          "Shows a Denom",
              PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "denom"}},
            },
            // this line is used by ignite scaffolding # autocli/query
          },
        },
        Tx: &autocliv1.ServiceCommandDescriptor{
          Service:              modulev1.Msg_ServiceDesc.ServiceName,
          EnhanceCustomCommand: true, // only required if you want to use the custom command
          RpcCommandOptions: []*autocliv1.RpcCommandOptions{
            {
              RpcMethod: "UpdateParams",
              Skip:      true, // skipped because authority gated
            },
            {
              RpcMethod:      "CreateDenom",
              Use:            "create-denom [denom] [description] [ticker] [precision] [url] [maxSupply] [canChangeMaxSupply]",
              Short:          "Create a new Denom",
              PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "denom"}, {ProtoField: "description"}, {ProtoField: "ticker"}, {ProtoField: "precision"}, {ProtoField: "url"}, {ProtoField: "maxSupply"}, {ProtoField: "canChangeMaxSupply"}},
            },
            {
              RpcMethod:      "UpdateDenom",
              Use:            "update-denom [denom] [description] [url] [maxSupply] [canChangeMaxSupply]",
              Short:          "Update Denom",
              PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "denom"}, {ProtoField: "description"}, {ProtoField: "url"}, {ProtoField: "maxSupply"}, {ProtoField: "canChangeMaxSupply"}},
            },
            {
              RpcMethod:      "DeleteDenom",
              Use:            "delete-denom [denom]",
              Short:          "Delete Denom",
              PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "denom"}},
            },
            {
              RpcMethod:      "MintTokens",
              Use:            "mint-tokens [denom] [amount] [recipient]",
              Short:          "Send a MintTokens tx",
              PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "denom"}, {ProtoField: "amount"}, {ProtoField: "recipient"}},
            },
            {
              RpcMethod:      "TransferTokens",
              Use:            "transfer-tokens [denom] [from] [to] [amount]",
              Short:          "Send a TransferTokens tx",
              PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "denom"}, {ProtoField: "from"}, {ProtoField: "to"}, {ProtoField: "amount"}},
            },
            // this line is used by ignite scaffolding # autocli/tx
          },
        },
      }
    }
  ```
## 5. Building and Serving Your Blockchain

- **Compile chain**:

  Compile your blockchain code by running:

  ```sh
    ignite chain build
  ```

  The successful execution of this command will be confirmed with a message indicating that the tokenfactory chain binary was built.

- **Serve chain**:

  To start your blockchain locally, use the following command:

  ```sh
    ignite chain serve
  ```

  This command launches a local blockchain node, compiles your code, and initializes its configuration. It also supports live reloading, so any changes you make to the code will be automatically applied without restarting the node.

  *Keep this terminal running as you proceed with the tests.*

## 6. Interact with Token Factory Blockchain

Congratulations on reaching the final stage! It's time to put your token factory module to the test

- **Creating a New Denom**:

In a new terminal, create a denom named uignite with the command:

  ```sh
    tokenfactoryd tx tokenmodule create-denom uignite "My denom" IGNITE 6 "some/url" 1000000000 true --from alice
  ```

- **Querying the Denom**:

Check the list of denoms to see your new creation:

  ```sh
    tokenfactoryd query tokenmodule list-denom
  ```

- **Updating the Denom**:

  - Modify the uignite denom:

  ```sh
    tokenfactoryd tx tokenmodule update-denom uignite "Ignite" "newurl" 2000000000 false --from alice
  ```

  - Query the denoms again to observe the changes:

  ```sh
    tokenfactoryd query tokenmodule list-denom
  ```

- **Minting and Sending Tokens**:

  - Mint uignite tokens and send them to a recipient:

    ```sh
      tokenfactoryd tx tokenmodule mint-tokens uignite 1200 <ALICE_ADDRESS> --from alice
    ```

  - Check the recipient’s balance:
    ```sh
      tokenfactoryd query bank balances <ALICE_ADDRESS>
    ```

  - Verify the updated supply in denom list:
    ```sh
      tokenfactoryd query tokenmodule list-denom
    ```

- **Transferring Tokens**:

  - Transfer some uignite tokens:

  ```sh
    tokenfactoryd tx tokenmodule transfer-tokens uignite <ALICE_ADDRESS> <BOB_ADDRESS> 100 --from alice
  ```

  - Check the recipient’s balance:
    ```sh
      tokenfactoryd query bank balances <BOB_ADDRESS>
    ```
  
- **Query transactions**

    - You can query transactions like so:

      ```sh
      tokenfactoryd query tx <TRANSACTION_HASH>
      ```

## Congratulations!

You've successfully built and tested a token factory module. This tutorial has equipped you with the skills to:

- Integrate Cosmos SDK modules and utilize their functionalities.
- Customize CRUD operations to fit your blockchain's needs.
- Scaffold modules and messages effectively.

## Resources

- [Go documentation](https://go.dev/doc/install).
- [Ignite CLI documentation](https://docs.ignite.com/nightly/welcome/install).
- [Code repository](github.com/cenwadike/)
