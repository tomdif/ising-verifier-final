package cmd

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

var defaultHome = os.ExpandEnv("$HOME/.nexusd")

func NewRootCmd() (*cobra.Command, error) {
	rootCmd := &cobra.Command{
		Use:   "nexusd",
		Short: "NEXUS - Drug Discovery Blockchain",
		Long: `NEXUS is a blockchain that combines Proof of Stake with Proof of Useful Work.
Validators secure transactions, miners perform molecular docking for drug discovery.

Dual Approval Consensus:
  Layer 1: Validators (Tendermint PoS) - Fast 3s blocks
  Layer 2: Miners (Proof of Useful Work) - Checkpoint every 200 blocks

To attack NEXUS requires BOTH 67% validator stake AND 51% miner compute.`,
	}

	rootCmd.AddCommand(
		InitCmd(),
		StartCmd(),
		StatusCmd(),
		MinerCmd(),
		CheckpointCmd(),
		QueryCmd(),
		TxCmd(),
	)

	rootCmd.PersistentFlags().String("home", defaultHome, "node home directory")

	return rootCmd, nil
}

func Execute() {
	rootCmd, err := NewRootCmd()
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	if err := rootCmd.Execute(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}
