import * as wasm from "palette-generator-wasm";
import {memory} from "palette-generator-wasm/palette_generator_wasm_bg.wasm";

window.onload = () => {
    const imagePreview = document.querySelector("#preview");
    const imageCanvas = document.querySelector("canvas");
    const imageCanvasContext = imageCanvas.getContext("2d");
    const imageCanvasData = imageCanvasContext.createImageData(
        imageCanvas.width,imageCanvas.width
        );
        const wasmImageArray = new Uint8Array(memory.buffer);
        var inputImageArray;
        imageCanvasContext.clearRect(0,0,imageCanvas.width,imageCanvas.height);
        const inputImage = document.querySelector("#input-image");
        inputImage.onchange = function (){
            //if (!this.file[0].type.match('image/png')){
            preview.src = URL.createObjectURL(this.files[0])
            var imageReader = new FileReader();
            imageReader.readAsDataURL(this.files[0]);
            imageReader.onload = function (){
                inputImageArray = new TextEncoder().encode(this.result);
                console.log(this.result);
                wasm.image(inputImageArray);
            }
        };
        
        document.ondrop = function(event){
            event.preventDefault();
            inputImage.files = event.dataTransfer.files;
            inputImage.onchange();
        }
        
        document.ondragover = function(event){
            event.preventDefault();
        }
        
        const generateButton = document.querySelector("#generate");
        generateButton.onclick = function()  {
            wasm.image_output(inputImageArray);
            const imagePointer = wasm.get_output_image_ptr();
            const wasmImageArray = new Uint8Array(memory.buffer);
            const imageDataArray = wasmImageArray.slice(imagePointer,imagePointer + 64*64*4)
            imageCanvasData.data.set(imageDataArray);
            imageCanvasContext.clearRect(0,0,imageCanvas.width,imageCanvas.height);
            imageCanvasContext.putImageData(imageCanvasData,0,0);  
        };
    }
    