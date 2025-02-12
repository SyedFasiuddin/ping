#include <windows.h>
#include <iphlpapi.h>
#include <ipmib.h>

#include <assert.h>
#include <stdio.h>

int main() {
    DWORD err;
    ULONG table_size = 0;

    /*
     * Query the size required for "IP table"
     */
    err = GetIpForwardTable(NULL, &table_size, FALSE);
    assert(err == ERROR_INSUFFICIENT_BUFFER && "GetIpForwardTable, we didn't allocate mem");

    /*
     * Allocate memory and get the IP table
     */
    PMIB_IPFORWARDTABLE table = calloc(table_size, 1);
    err = GetIpForwardTable(table, &table_size, FALSE);
    assert(err == NO_ERROR && "GetIpForwardTable, should have been NO_ERROR");

    /*
     * Find the index of default interface in returned IP table array
     */
    int ifaceIdx = -1;
    for (int i = 0; i < table->dwNumEntries; i++) {
        if (table->table[i].dwForwardDest == 0) {   // Dest: "0.0.0.0"
            ifaceIdx = table->table[i].dwForwardIfIndex;
            break;
        }
    }
    assert(ifaceIdx != -1 && "Default interface not found");

    /*
     * Query the size required for "Interface info"
     */
    err = GetInterfaceInfo(NULL, &table_size);
    assert(err == ERROR_INSUFFICIENT_BUFFER && "GetInterfaceInfo, we didn't allocate mem");

    /*
     * Allocate memory and get the "Interface information"
     */
    PIP_INTERFACE_INFO interfaces = calloc(table_size, 1);
    err = GetInterfaceInfo(interfaces, &table_size);
    assert(err == NO_ERROR && "GetInterfaceInfo, should have been NO_ERROR");

    /*
     * Find the interface with index we got from IP Table in Interface information
     */
    int ifaceInfoIdx = -1;
    for (int i = 0; i < interfaces->NumAdapters; i++) {
        if (interfaces->Adapter[i].Index == ifaceIdx) {
            ifaceInfoIdx = i;
            break;
        }
    }
    assert(ifaceInfoIdx != -1 && "No information about default interface");

    printf("Found it! Name is: %S\n", interfaces->Adapter[ifaceInfoIdx].Name);

    return 0;
}
