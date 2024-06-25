const ws: WebSocket = new WebSocket('ws://localhost:3000');
const canvas: HTMLCanvasElement = document.querySelector("canvas")!;
const ctx: CanvasRenderingContext2D = canvas.getContext("2d")!;

canvas.width = 256;
canvas.height = 256;

const COLORS = ["red", "blue", "green", "yellow", "purple"];

let buf: Uint8Array;

ws.binaryType = "arraybuffer";

ws.onopen = () => {
    console.log("connected");
    render();
}

ws.onmessage = (e: MessageEvent) => {
    const arr_buf: ArrayBuffer = e.data;
    buf = new Uint8Array(arr_buf);
    //const buf: Uint8Array = new Uint8Array(arr_buf);
    //render(buf);
}

document.onkeydown = (e: KeyboardEvent) => {

    if(!ws.OPEN || e.repeat) {
        return;
    }

    switch(e.key) {
        case("w"):
            send_client_event(ws, 0);
            break;
        case("a"):
            send_client_event(ws, 2);
            break;
        case("d"):
            send_client_event(ws, 4);
            break;
    }
    
};

document.onkeyup = (e: KeyboardEvent) => {

    if(!ws.OPEN) {
        return;
    }

    switch(e.key) {
        case("w"):
            send_client_event(ws, 1);
            break;
        case("a"):
            send_client_event(ws, 3);
            break;
        case("d"):
            send_client_event(ws, 5);
            break;
    }
    
};

function render() {

    if(!buf) {
        return requestAnimationFrame(render);
    }

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // need to parse header eventually, but there is none for now
    for(let i = 0; i + 4 < buf.length; i += 5) {

        const color_idx: number = buf[i];

        const x: number = buf[i+1];
        const y: number = buf[i+2];

        const width: number = buf[i+3];
        const height: number = buf[i+4];

        ctx.fillStyle = COLORS[color_idx];
        ctx.fillRect(x, y, width, height);

    }

    requestAnimationFrame(render);

}

function send_client_event(ws: WebSocket, byte: number) {
    const buf: Uint8Array = new Uint8Array(1);
    buf[0] = byte;
    ws.send(buf);
}

// function render(buf: Uint8Array) {

// }