cmake_minimum_required(VERSION 3.1)

project(macrokey VERSION 1.0 LANGUAGES CXX)

# Set a default build type if none was specified
set(default_build_type "Release")
if(EXISTS "${CMAKE_SOURCE_DIR}/.git")
  set(default_build_type "Debug")
endif()

# enable debug via cmd
# cmake -DDEFINE_DEBUG=ON -DCMAKE_INSTALL_PREFIX=/usr ..
option(DEFINE_DEBUG
  "Build using debug"
  OFF)

if(DEFINE_DEBUG)
  message("Adding Debug flag...")
  add_definitions(-DDEBUG)
  set(CMAKE_BUILD_TYPE Debug)
  message("Build type is " ${CMAKE_BUILD_TYPE})
endif(DEFINE_DEBUG)


# set output directories
#set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
#set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
#set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

set(CMAKE_INCLUDE_CURRENT_DIR ON)
set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# libraries
find_package(Boost COMPONENTS python REQUIRED)
message("Include dirs of boost: " ${Boost_INCLUDE_DIRS} )
message("Libs of boost: " ${Boost_LIBRARIES} )

find_package(PythonLibs REQUIRED)
message("Include dirs of Python: " ${PYTHON_INCLUDE_DIRS} )
message("Libs of Python: " ${PYTHON_LIBRARIES} )


include_directories(
    ${Boost_INCLUDE_DIRS}
    ${PYTHON_INCLUDE_DIRS}
)

set(SOURCES
    src/main.cpp
    src/uhid_device.cpp
)

set(HEADERS
    src/uhid_device.h
)


if(CMAKE_BUILD_TYPE STREQUAL "Debug")

    # add executable
    # use this for testing
    add_executable(
        ${PROJECT_NAME}
        ${SOURCES}
        ${HEADERS}
        ${APP_RESOURCES}    # App resources file
    )

else()

    # shared library .so file
    ADD_LIBRARY( ${PROJECT_NAME} SHARED
        ${SOURCES} 
        ${HEADERS} 
        ${RESOURCE_FILES}  
        ${CMAKE_HELPER_FILES} )

endif(CMAKE_BUILD_TYPE STREQUAL "Debug")


target_link_libraries(
    ${PROJECT_NAME}
    ${Boost_LIBRARIES}
    ${PYTHON_LIBRARIES}
)