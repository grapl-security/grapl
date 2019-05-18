import variables from '../../scss/variables/_breakpoints.scss';

Object.keys(variables).forEach((key) => {
  variables[key] = parseInt(variables[key].slice(0, -2), 10);
});

export default variables;
