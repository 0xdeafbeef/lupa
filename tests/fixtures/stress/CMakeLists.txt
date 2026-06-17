cmake_minimum_required(VERSION 3.24)
project(Stress LANGUAGES C CXX)

function(add_stress_target name)
  if(ARGN)
    foreach(src IN LISTS ARGN)
      target_sources(${name} PRIVATE ${src})
    endforeach()
  endif()
endfunction()

macro(enable_feature target feature)
  target_compile_definitions(${target} PRIVATE "FEATURE_${feature}=1")
endmacro()
