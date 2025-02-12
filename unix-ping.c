#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/socket.h>
#include <netinet/ip.h>
#include <netinet/ip_icmp.h>
#include <arpa/inet.h>

uint16_t
checksum (void *addr, size_t len) {
    uint16_t *word = addr;
    uint32_t result = 0;
    for (size_t i = 0; i < len / sizeof(uint16_t); i++) {
        result += *(word + i);
    }
    if (len % 2 == 1) {
        result += *((uint8_t *)addr + len - 1);
    }
    result = (result >> 16) + (result & 0xffff);
    // Carry the previous add.
    result = (result >> 16) + (result & 0xffff);
    return ~result;
}

int
main (void)
{
    // To get the protocol number from name of protocol. Ref: /etc/protcols
    // struct protoent entry = *getprotobyname ("ICMP");
    // printf ("Official Name: %s\nProtocol Num: %d\n", entry.p_name, entry.p_proto);
    // socket (AF_INET, SOCK_DGRAM, entry.p_proto);

    /*
     * Prepare ICMP packet
     */
    const char *data = "Hello World";
    uint8_t packet[64];
    memset (packet, 0, sizeof(packet));

    struct icmp *icmp_msg;
    icmp_msg = (struct icmp*)packet;
    icmp_msg->icmp_type = ICMP_ECHO;
    icmp_msg->icmp_code = 0;
    icmp_msg->icmp_id = 69;
    icmp_msg->icmp_seq = 420;

    memcpy (packet + sizeof(struct icmp), data, strlen(data));
    icmp_msg->icmp_cksum = checksum (packet, sizeof(struct icmp) + strlen(data));
    printf ("SEQ: %d\n", icmp_msg->icmp_seq);
    // __builtin_dump_struct (icmp_msg, &printf);

    /*
     * Prepare a destination address
     */
    // struct in_addr dest_ip = {0};
    // Note: no error checking since we are providing valid IP, also there is
    // no mention of return value (-1) on failure.
    // inet_aton ("8.8.8.8", &dest_ip);
    struct sockaddr_in dest = {
        .sin_family = AF_INET,
        .sin_port = 0,
        .sin_addr.s_addr = inet_addr ("8.8.8.8"),
    };

    // char data[1000];
    // memcpy (data, &icmp_msg, sizeof(icmp_msg));
    // memcpy (data + sizeof(icmp_msg), "Hello World", 11);

    /*
     * Send that packet
     * NOTE: no need to bind
     */
    int socket_fd = socket (AF_INET, SOCK_DGRAM, IPPROTO_ICMP);
    if (socket_fd == -1) {
        perror("socket");
        exit(EXIT_FAILURE);
    }
    int bytes_sent = sendto (socket_fd,                  // socket
            &packet,                  // buffer
            sizeof(packet),           // buffer len
            0,                          // flags
            (struct sockaddr*) &dest,   // dest_addr
            sizeof(dest)                // dest_addr len
            );
    printf ("Bytes sent: %d\n", bytes_sent);

    /*
     * Recieve a reply
     */
    memset (packet, 0, sizeof(packet));
    socklen_t dest_size = sizeof(dest);
    int bytes_read = recvfrom (socket_fd,                // socket
                               packet,                   // recv buffer
                               sizeof(packet),           // recv buffer len
                               0,                        // flags
                               (struct sockaddr*)&dest, &dest_size);              // Perr addr return
    // __builtin_dump_struct (icmp_msg, &printf);
    // skip the ip header, this really confused me
    icmp_msg = (struct icmp*)(packet + sizeof(struct ip));
    printf ("Bytes Read: %d\n\n", bytes_read);
    printf ("icmp_type: %d\n", icmp_msg->icmp_type);
    printf ("icmp_id: %d\n", icmp_msg->icmp_id);
    printf ("icmp_sed: %d\n", icmp_msg->icmp_seq);
    printf ("icmp_data: %s\n", packet + sizeof(struct ip) + sizeof(struct icmp));

    close (socket_fd);

    return 0;
}
