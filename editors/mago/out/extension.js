"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode_1 = require("vscode");
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
    let executable = {
        command: '/Users/azjezz/mago/mago/target/debug/mago',
        args: ['lsp'],
    };
    let serverOptions = {
        run: executable,
        debug: executable,
    };
    let clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'php' }],
        synchronize: {
            fileEvents: vscode_1.workspace.createFileSystemWatcher('**/*.php')
        },
    };
    client = new node_1.LanguageClient('mago', 'Mago Language Server', serverOptions, clientOptions);
    // Start the client. This will also launch the server.
    client.start();
}
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
//# sourceMappingURL=extension.js.map