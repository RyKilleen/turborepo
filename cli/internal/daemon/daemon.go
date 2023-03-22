package daemon

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/hashicorp/go-hclog"
	"github.com/pkg/errors"
	"github.com/vercel/turbo/cli/internal/daemon/connector"
	"github.com/vercel/turbo/cli/internal/fs"
	"github.com/vercel/turbo/cli/internal/server"
	"github.com/vercel/turbo/cli/internal/turbopath"
)

func getRepoHash(repoRoot turbopath.AbsoluteSystemPath) string {
	pathHash := sha256.Sum256([]byte(repoRoot.ToString()))
	// We grab a substring of the hash because there is a 108-character limit on the length
	// of a filepath for unix domain socket.
	return hex.EncodeToString(pathHash[:])[:16]
}

func getDaemonFileRoot(repoRoot turbopath.AbsoluteSystemPath) turbopath.AbsoluteSystemPath {
	tempDir := fs.TempDir("turbod")
	hexHash := getRepoHash(repoRoot)
	return tempDir.UntypedJoin(hexHash)
}

func getLogFilePath(repoRoot turbopath.AbsoluteSystemPath) (turbopath.AbsoluteSystemPath, error) {
	hexHash := getRepoHash(repoRoot)
	base := repoRoot.Base()
	logFilename := fmt.Sprintf("%v-%v.log", hexHash, base)

	logsDir := fs.GetTurboDataDir().UntypedJoin("logs")
	return logsDir.UntypedJoin(logFilename), nil
}

func getUnixSocket(repoRoot turbopath.AbsoluteSystemPath) turbopath.AbsoluteSystemPath {
	root := getDaemonFileRoot(repoRoot)
	return root.UntypedJoin("turbod.sock")
}

func getPidFile(repoRoot turbopath.AbsoluteSystemPath) turbopath.AbsoluteSystemPath {
	root := getDaemonFileRoot(repoRoot)
	return root.UntypedJoin("turbod.pid")
}

// we're only appending, and we're creating the file if it doesn't exist.
// we do not need to read the log file.
var _logFileFlags = os.O_WRONLY | os.O_APPEND | os.O_CREATE

var errInactivityTimeout = errors.New("turbod shut down from inactivity")

type rpcServer interface {
	Register(grpcServer server.GRPCServer)
}

// ClientOpts re-exports connector.Ops to encapsulate the connector package
type ClientOpts = connector.Opts

// Client re-exports connector.Client to encapsulate the connector package
type Client = connector.Client

// GetClient returns a client that can be used to interact with the daemon
func GetClient(ctx context.Context, repoRoot turbopath.AbsoluteSystemPath, logger hclog.Logger, turboVersion string, opts ClientOpts) (*Client, error) {
	sockPath := getUnixSocket(repoRoot)
	pidPath := getPidFile(repoRoot)
	logPath, err := getLogFilePath(repoRoot)
	if err != nil {
		return nil, err
	}
	bin, err := os.Executable()
	if err != nil {
		return nil, err
	}
	// The Go binary can no longer be called directly, so we need to route back to the rust wrapper
	if strings.HasSuffix(bin, "go-turbo") {
		bin = filepath.Join(filepath.Dir(bin), "turbo")
	} else if strings.HasSuffix(bin, "go-turbo.exe") {
		bin = filepath.Join(filepath.Dir(bin), "turbo.exe")
	}
	c := &connector.Connector{
		Logger:       logger.Named("TurbodClient"),
		Bin:          bin,
		Opts:         opts,
		SockPath:     sockPath,
		PidPath:      pidPath,
		LogPath:      logPath,
		TurboVersion: turboVersion,
	}
	return c.Connect(ctx)
}
