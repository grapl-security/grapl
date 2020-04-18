import React from 'react';
import ReactDOM from 'react-dom';
import './index.css';
import { createMuiTheme, ThemeProvider } from '@material-ui/core/styles'
import App from './App';
import * as serviceWorker from './serviceWorker';


const darkTheme = createMuiTheme({
    palette: {
        type: 'dark',
        primary: {
            main: '#373740',
        }
    }
})

const rootElement = document.getElementById('root')

ReactDOM.render(
    <React.StrictMode>
    <ThemeProvider theme={darkTheme}>
        <App />, 
    </ThemeProvider>
    </React.StrictMode>
    ,
    rootElement
);

serviceWorker.unregister();
